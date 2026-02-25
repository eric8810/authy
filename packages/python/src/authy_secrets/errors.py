"""Typed exception hierarchy for authy CLI errors."""

from __future__ import annotations


class AuthyError(Exception):
    """Base error with exit_code and error_code from authy CLI."""

    def __init__(self, exit_code: int, error_code: str, message: str) -> None:
        self.exit_code = exit_code
        self.error_code = error_code
        self.message = message
        super().__init__(message)


class SecretNotFound(AuthyError):
    """Raised when a requested secret does not exist (exit code 3, code not_found)."""


class SecretAlreadyExists(AuthyError):
    """Raised when storing a secret that already exists without --force (exit code 5)."""


class AuthFailed(AuthyError):
    """Raised when authentication fails (exit code 2)."""


class PolicyDenied(AuthyError):
    """Raised when access is denied by a policy (exit code 4)."""


class VaultNotFound(AuthyError):
    """Raised when the vault is not initialized (exit code 7)."""


# Mapping from (exit_code) to exception class.
# Some exit codes map to multiple error_codes; we use the exit code as the
# primary discriminator and fall back to the base AuthyError for unknown codes.
_EXIT_CODE_MAP: dict[int, type[AuthyError]] = {
    2: AuthFailed,
    3: SecretNotFound,
    4: PolicyDenied,
    5: SecretAlreadyExists,
    7: VaultNotFound,
}


def _map_error(error: dict, exit_code: int) -> AuthyError:
    """Map a parsed JSON error dict + exit code to a typed exception."""
    code = error.get("code", "unknown")
    message = error.get("message", "Unknown error")
    cls = _EXIT_CODE_MAP.get(exit_code, AuthyError)
    return cls(exit_code=exit_code, error_code=code, message=message)
