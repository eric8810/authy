"""Unit tests for authy_secrets.Authy — all subprocess calls are mocked."""

from __future__ import annotations

import json
import subprocess
from unittest.mock import MagicMock, patch

import pytest

from authy_secrets import (
    Authy,
    AuthFailed,
    AuthyError,
    PolicyDenied,
    SecretAlreadyExists,
    SecretNotFound,
    VaultNotFound,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _completed(
    stdout: str = "",
    stderr: str = "",
    returncode: int = 0,
) -> subprocess.CompletedProcess:
    return subprocess.CompletedProcess(
        args=[], returncode=returncode, stdout=stdout, stderr=stderr
    )


def _json_stdout(data: dict) -> subprocess.CompletedProcess:
    return _completed(stdout=json.dumps(data))


def _json_error(code: str, message: str, exit_code: int) -> subprocess.CompletedProcess:
    body = {"error": {"code": code, "message": message, "exit_code": exit_code}}
    return _completed(stderr=json.dumps(body), returncode=exit_code)


# ---------------------------------------------------------------------------
# Construction
# ---------------------------------------------------------------------------

class TestConstruction:
    @patch("authy_secrets.client.shutil.which", return_value="/usr/bin/authy")
    def test_finds_binary_on_path(self, mock_which: MagicMock) -> None:
        client = Authy()
        assert client._binary == "/usr/bin/authy"
        mock_which.assert_called_once_with("authy")

    def test_custom_binary_path(self) -> None:
        client = Authy(binary="/opt/authy/bin/authy")
        assert client._binary == "/opt/authy/bin/authy"

    @patch("authy_secrets.client.shutil.which", return_value=None)
    def test_raises_when_binary_not_found(self, mock_which: MagicMock) -> None:
        with pytest.raises(FileNotFoundError, match="authy binary not found"):
            Authy()

    def test_credentials_in_env(self) -> None:
        client = Authy(binary="/bin/authy", passphrase="s3cret")
        assert client._extra_env["AUTHY_PASSPHRASE"] == "s3cret"

        client2 = Authy(binary="/bin/authy", keyfile="/path/to/key")
        assert client2._extra_env["AUTHY_KEYFILE"] == "/path/to/key"


# ---------------------------------------------------------------------------
# get / get_or_none
# ---------------------------------------------------------------------------

class TestGet:
    @patch("authy_secrets.client.subprocess.run")
    def test_get_returns_value(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "name": "db-url",
            "value": "postgres://localhost/mydb",
            "version": 1,
            "created": "2025-01-01T00:00:00Z",
            "modified": "2025-01-01T00:00:00Z",
        })
        client = Authy(binary="/bin/authy")
        assert client.get("db-url") == "postgres://localhost/mydb"

        # Verify the CLI was called with --json get <name>
        call_args = mock_run.call_args
        assert call_args[0][0] == ["/bin/authy", "--json", "get", "db-url"]

    @patch("authy_secrets.client.subprocess.run")
    def test_get_not_found_raises(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error("not_found", "Secret not found: db-url", 3)
        client = Authy(binary="/bin/authy")
        with pytest.raises(SecretNotFound) as exc_info:
            client.get("db-url")
        assert exc_info.value.exit_code == 3
        assert exc_info.value.error_code == "not_found"

    @patch("authy_secrets.client.subprocess.run")
    def test_get_or_none_returns_none(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error("not_found", "Secret not found: db-url", 3)
        client = Authy(binary="/bin/authy")
        assert client.get_or_none("db-url") is None

    @patch("authy_secrets.client.subprocess.run")
    def test_get_or_none_returns_value(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "name": "db-url",
            "value": "postgres://localhost/mydb",
            "version": 1,
            "created": "2025-01-01T00:00:00Z",
            "modified": "2025-01-01T00:00:00Z",
        })
        client = Authy(binary="/bin/authy")
        assert client.get_or_none("db-url") == "postgres://localhost/mydb"


# ---------------------------------------------------------------------------
# store
# ---------------------------------------------------------------------------

class TestStore:
    @patch("authy_secrets.client.subprocess.run")
    def test_store_passes_value_via_stdin(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _completed()
        client = Authy(binary="/bin/authy")
        client.store("api-key", "sk-1234")

        call_args = mock_run.call_args
        assert call_args[0][0] == ["/bin/authy", "--json", "store", "api-key"]
        assert call_args[1]["input"] == "sk-1234"

    @patch("authy_secrets.client.subprocess.run")
    def test_store_with_force(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _completed()
        client = Authy(binary="/bin/authy")
        client.store("api-key", "sk-5678", force=True)

        call_args = mock_run.call_args
        assert call_args[0][0] == ["/bin/authy", "--json", "store", "api-key", "--force"]

    @patch("authy_secrets.client.subprocess.run")
    def test_store_duplicate_raises(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error(
            "already_exists",
            "Secret already exists: api-key (use --force to overwrite)",
            5,
        )
        client = Authy(binary="/bin/authy")
        with pytest.raises(SecretAlreadyExists) as exc_info:
            client.store("api-key", "sk-1234")
        assert exc_info.value.exit_code == 5


# ---------------------------------------------------------------------------
# remove
# ---------------------------------------------------------------------------

class TestRemove:
    @patch("authy_secrets.client.subprocess.run")
    def test_remove_returns_true(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _completed()
        client = Authy(binary="/bin/authy")
        assert client.remove("old-secret") is True

    @patch("authy_secrets.client.subprocess.run")
    def test_remove_not_found_raises(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error("not_found", "Secret not found: old-secret", 3)
        client = Authy(binary="/bin/authy")
        with pytest.raises(SecretNotFound):
            client.remove("old-secret")


# ---------------------------------------------------------------------------
# rotate
# ---------------------------------------------------------------------------

class TestRotate:
    @patch("authy_secrets.client.subprocess.run")
    def test_rotate_returns_version(self, mock_run: MagicMock) -> None:
        # First call: rotate (no stdout)
        # Second call: get (returns version)
        mock_run.side_effect = [
            _completed(),
            _json_stdout({
                "name": "api-key",
                "value": "new-val",
                "version": 2,
                "created": "2025-01-01T00:00:00Z",
                "modified": "2025-01-02T00:00:00Z",
            }),
        ]
        client = Authy(binary="/bin/authy")
        version = client.rotate("api-key", "new-val")
        assert version == 2

        # Verify rotate was called with stdin
        rotate_call = mock_run.call_args_list[0]
        assert rotate_call[0][0] == ["/bin/authy", "--json", "rotate", "api-key"]
        assert rotate_call[1]["input"] == "new-val"


# ---------------------------------------------------------------------------
# list
# ---------------------------------------------------------------------------

class TestList:
    @patch("authy_secrets.client.subprocess.run")
    def test_list_returns_names(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "secrets": [
                {"name": "db-url", "version": 1, "created": "2025-01-01T00:00:00Z", "modified": "2025-01-01T00:00:00Z"},
                {"name": "api-key", "version": 3, "created": "2025-01-01T00:00:00Z", "modified": "2025-01-02T00:00:00Z"},
            ]
        })
        client = Authy(binary="/bin/authy")
        names = client.list()
        assert names == ["db-url", "api-key"]

    @patch("authy_secrets.client.subprocess.run")
    def test_list_with_scope(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "secrets": [
                {"name": "db-url", "version": 1, "created": "2025-01-01T00:00:00Z", "modified": "2025-01-01T00:00:00Z"},
            ]
        })
        client = Authy(binary="/bin/authy")
        names = client.list(scope="deploy")
        call_args = mock_run.call_args
        assert call_args[0][0] == ["/bin/authy", "--json", "list", "--scope", "deploy"]

    @patch("authy_secrets.client.subprocess.run")
    def test_list_empty(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({"secrets": []})
        client = Authy(binary="/bin/authy")
        assert client.list() == []


# ---------------------------------------------------------------------------
# auth errors
# ---------------------------------------------------------------------------

class TestAuthErrors:
    @patch("authy_secrets.client.subprocess.run")
    def test_auth_failed_raises(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error(
            "auth_failed", "Authentication failed: bad passphrase", 2
        )
        client = Authy(binary="/bin/authy")
        with pytest.raises(AuthFailed) as exc_info:
            client.get("db-url")
        assert exc_info.value.exit_code == 2

    @patch("authy_secrets.client.subprocess.run")
    def test_policy_denied_raises(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error(
            "access_denied", "Access denied: secret 'x' not allowed by scope 'y'", 4
        )
        client = Authy(binary="/bin/authy")
        with pytest.raises(PolicyDenied) as exc_info:
            client.get("x")
        assert exc_info.value.exit_code == 4

    @patch("authy_secrets.client.subprocess.run")
    def test_vault_not_found_raises(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_error(
            "vault_not_initialized", "Vault not initialized. Run `authy init` first.", 7
        )
        client = Authy(binary="/bin/authy")
        with pytest.raises(VaultNotFound) as exc_info:
            client.get("db-url")
        assert exc_info.value.exit_code == 7


# ---------------------------------------------------------------------------
# credentials env forwarding
# ---------------------------------------------------------------------------

class TestCredentialsEnv:
    @patch("authy_secrets.client.subprocess.run")
    def test_passphrase_forwarded_to_subprocess(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "name": "x",
            "value": "v",
            "version": 1,
            "created": "2025-01-01T00:00:00Z",
            "modified": "2025-01-01T00:00:00Z",
        })
        client = Authy(binary="/bin/authy", passphrase="s3cret")
        client.get("x")

        call_env = mock_run.call_args[1]["env"]
        assert call_env["AUTHY_PASSPHRASE"] == "s3cret"

    @patch("authy_secrets.client.subprocess.run")
    def test_keyfile_forwarded_to_subprocess(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "name": "x",
            "value": "v",
            "version": 1,
            "created": "2025-01-01T00:00:00Z",
            "modified": "2025-01-01T00:00:00Z",
        })
        client = Authy(binary="/bin/authy", keyfile="/path/to/key")
        client.get("x")

        call_env = mock_run.call_args[1]["env"]
        assert call_env["AUTHY_KEYFILE"] == "/path/to/key"


# ---------------------------------------------------------------------------
# context manager
# ---------------------------------------------------------------------------

class TestContextManager:
    @patch("authy_secrets.client.subprocess.run")
    def test_context_manager(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _json_stdout({
            "name": "x",
            "value": "v",
            "version": 1,
            "created": "2025-01-01T00:00:00Z",
            "modified": "2025-01-01T00:00:00Z",
        })
        with Authy(binary="/bin/authy", keyfile="/path/to/key") as client:
            val = client.get("x")
        assert val == "v"


# ---------------------------------------------------------------------------
# unparseable stderr
# ---------------------------------------------------------------------------

class TestUnparseableError:
    @patch("authy_secrets.client.subprocess.run")
    def test_unparseable_stderr_raises_base_error(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _completed(
            stderr="something went wrong", returncode=1
        )
        client = Authy(binary="/bin/authy")
        with pytest.raises(AuthyError) as exc_info:
            client.get("x")
        assert exc_info.value.exit_code == 1
        assert "something went wrong" in exc_info.value.message


# ---------------------------------------------------------------------------
# import_dotenv
# ---------------------------------------------------------------------------

class TestImportDotenv:
    @patch("authy_secrets.client.subprocess.run")
    def test_import_dotenv(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _completed()
        client = Authy(binary="/bin/authy")
        client.import_dotenv(".env")
        call_args = mock_run.call_args
        assert call_args[0][0] == ["/bin/authy", "--json", "import", ".env"]

    @patch("authy_secrets.client.subprocess.run")
    def test_import_dotenv_force(self, mock_run: MagicMock) -> None:
        mock_run.return_value = _completed()
        client = Authy(binary="/bin/authy")
        client.import_dotenv(".env", force=True)
        call_args = mock_run.call_args
        assert call_args[0][0] == ["/bin/authy", "--json", "import", ".env", "--force"]


# ---------------------------------------------------------------------------
# is_initialized (static)
# ---------------------------------------------------------------------------

class TestIsInitialized:
    @patch("authy_secrets.client.shutil.which", return_value="/bin/authy")
    @patch("authy_secrets.client.subprocess.run")
    def test_returns_true_when_vault_exists(
        self, mock_run: MagicMock, mock_which: MagicMock
    ) -> None:
        # Any error other than vault_not_initialized means vault exists
        mock_run.return_value = _json_error("auth_failed", "Authentication failed", 2)
        assert Authy.is_initialized() is True

    @patch("authy_secrets.client.shutil.which", return_value="/bin/authy")
    @patch("authy_secrets.client.subprocess.run")
    def test_returns_false_when_no_vault(
        self, mock_run: MagicMock, mock_which: MagicMock
    ) -> None:
        mock_run.return_value = _json_error(
            "vault_not_initialized",
            "Vault not initialized. Run `authy init` first.",
            7,
        )
        assert Authy.is_initialized() is False

    @patch("authy_secrets.client.shutil.which", return_value=None)
    def test_raises_when_binary_not_found(self, mock_which: MagicMock) -> None:
        with pytest.raises(FileNotFoundError):
            Authy.is_initialized()
