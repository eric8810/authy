"""Authy CLI — native Python binding for the authy secrets manager.

Usage::

    from authy_cli import Authy

    client = Authy(passphrase="my-vault-passphrase")
    client.store("api-key", "sk-secret-value")
    value = client.get("api-key")
"""

from authy_cli._native import (
    Authy,
    AuthyException as AuthyError,
    SecretNotFound,
    SecretAlreadyExists,
    AuthFailed,
    PolicyNotFound,
    AccessDenied,
    VaultNotInitialized,
)

__all__ = [
    "Authy",
    "AuthyError",
    "SecretNotFound",
    "SecretAlreadyExists",
    "AuthFailed",
    "PolicyNotFound",
    "AccessDenied",
    "VaultNotInitialized",
]
