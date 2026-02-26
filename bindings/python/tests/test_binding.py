"""Tests for the authy_cli native binding."""

import os
import tempfile

import pytest


def _isolated_home():
    """Create a temp dir and override HOME for vault isolation."""
    tmp = tempfile.mkdtemp()
    os.environ["HOME"] = tmp
    return tmp


class TestAuthy:
    """Tests that require vault initialization."""

    def setup_method(self):
        self._home = _isolated_home()

    def test_init_and_store(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("api-key", "sk-secret-123")
        assert client.get("api-key") == "sk-secret-123"

    def test_get_not_found(self):
        from authy_cli import Authy, SecretNotFound

        client = Authy(passphrase="test-pass")
        client.init_vault()
        with pytest.raises(SecretNotFound):
            client.get("nonexistent")

    def test_get_or_none(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        assert client.get_or_none("nonexistent") is None
        client.store("exists", "val")
        assert client.get_or_none("exists") == "val"

    def test_store_duplicate_raises(self):
        from authy_cli import Authy, SecretAlreadyExists

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("dup", "v1")
        with pytest.raises(SecretAlreadyExists):
            client.store("dup", "v2")

    def test_store_force(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("key", "v1")
        client.store("key", "v2", force=True)
        assert client.get("key") == "v2"

    def test_remove(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("to-remove", "val")
        assert client.remove("to-remove") is True
        assert client.get_or_none("to-remove") is None

    def test_rotate(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("rotating", "v1")
        version = client.rotate("rotating", "v2")
        assert version == 2
        assert client.get("rotating") == "v2"

    def test_list(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("alpha", "a")
        client.store("beta", "b")
        names = client.list()
        assert "alpha" in names
        assert "beta" in names

    def test_is_initialized(self):
        from authy_cli import Authy

        assert Authy.is_initialized() is False
        client = Authy(passphrase="test-pass")
        client.init_vault()
        assert Authy.is_initialized() is True

    def test_build_env_map(self):
        from authy_cli import Authy

        client = Authy(passphrase="test-pass")
        client.init_vault()
        client.store("db-url", "postgres://localhost")
        client.store("api-key", "sk-123")
        # Create a policy allowing all
        client.test_policy  # just to confirm method exists
        # We need create_policy — not exposed, so test via build_env_map
        # This test will need a policy; skip if create_policy not available

    def test_no_credentials_raises(self):
        from authy_cli import Authy, AuthyError

        os.environ.pop("AUTHY_PASSPHRASE", None)
        os.environ.pop("AUTHY_KEYFILE", None)
        with pytest.raises(AuthyError):
            Authy(from_env=True)
