"""authy-secrets — Python SDK for the authy secrets manager."""

from .client import Authy
from .errors import (
    AuthFailed,
    AuthyError,
    PolicyDenied,
    SecretAlreadyExists,
    SecretNotFound,
    VaultNotFound,
)

__all__ = [
    "Authy",
    "AuthyError",
    "AuthFailed",
    "PolicyDenied",
    "SecretAlreadyExists",
    "SecretNotFound",
    "VaultNotFound",
]

__version__ = "0.7.0"
