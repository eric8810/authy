"""authy-secrets — Python SDK for the authy secrets manager.

DEPRECATED: This package has been replaced by ``authy-cli``, which includes
native Rust bindings via PyO3.  Install with: pip install authy-cli
"""

import warnings as _warnings
_warnings.warn(
    "authy-secrets is deprecated. Install authy-cli instead: pip install authy-cli",
    DeprecationWarning,
    stacklevel=2,
)

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

__version__ = "0.7.1"
