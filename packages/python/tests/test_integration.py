"""Integration tests — require a real ``authy`` binary on PATH.

Run with:  pytest -m integration
Skip reason: these tests are automatically skipped if ``authy`` is not found.
"""

from __future__ import annotations

import os
import shutil
import tempfile

import pytest

from authy_secrets import Authy, SecretAlreadyExists, SecretNotFound

# Skip the entire module if authy is not available
pytestmark = pytest.mark.integration
_AUTHY_BIN = shutil.which("authy")


def _skip_if_no_authy() -> None:
    if _AUTHY_BIN is None:
        pytest.skip("authy binary not found on PATH")


@pytest.fixture(autouse=True)
def check_authy() -> None:
    _skip_if_no_authy()


@pytest.fixture()
def isolated_vault(tmp_path):
    """Create a temporary HOME so authy init creates a fresh vault."""
    old_home = os.environ.get("HOME")
    os.environ["HOME"] = str(tmp_path)
    # Also clear any existing authy env vars
    for key in ("AUTHY_PASSPHRASE", "AUTHY_KEYFILE", "AUTHY_TOKEN"):
        os.environ.pop(key, None)
    # Set a deterministic passphrase for test vault
    os.environ["AUTHY_PASSPHRASE"] = "test-passphrase"
    yield tmp_path
    # Restore
    if old_home is not None:
        os.environ["HOME"] = old_home
    else:
        os.environ.pop("HOME", None)
    os.environ.pop("AUTHY_PASSPHRASE", None)


class TestRoundtrip:
    def test_init_store_get_roundtrip(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()

        client.store("db-url", "postgres://localhost/testdb")
        value = client.get("db-url")
        assert value == "postgres://localhost/testdb"

    def test_store_duplicate_raises(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()
        client.store("api-key", "sk-first")

        with pytest.raises(SecretAlreadyExists):
            client.store("api-key", "sk-second")

    def test_store_force_overwrites(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()
        client.store("api-key", "sk-first")
        client.store("api-key", "sk-second", force=True)
        assert client.get("api-key") == "sk-second"

    def test_remove(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()
        client.store("temp-secret", "value")
        assert client.remove("temp-secret") is True

        with pytest.raises(SecretNotFound):
            client.get("temp-secret")

    def test_rotate_bumps_version(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()
        client.store("rotating-key", "v1")
        version = client.rotate("rotating-key", "v2")
        assert version == 2
        assert client.get("rotating-key") == "v2"

    def test_list(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()
        client.store("alpha", "a")
        client.store("beta", "b")
        names = client.list()
        assert "alpha" in names
        assert "beta" in names

    def test_get_or_none_missing(self, isolated_vault) -> None:
        client = Authy(passphrase="test-passphrase")
        client.init()
        assert client.get_or_none("nonexistent") is None

    def test_is_initialized(self, isolated_vault) -> None:
        assert Authy.is_initialized() is False
        client = Authy(passphrase="test-passphrase")
        client.init()
        assert Authy.is_initialized() is True
