"""Authy CLI wrapper — thin subprocess-based Python SDK."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from typing import Any, Dict, List, Optional

from .errors import AuthyError, _map_error


class Authy:
    """Python client for the authy secrets manager.

    Wraps the ``authy`` CLI binary, using ``--json`` output for structured
    communication. Secret values are always passed via stdin, never as
    command-line arguments.

    Can be used as a context manager to scope credential environment variables::

        with Authy(keyfile="/path/to/key") as client:
            client.get("db-url")
    """

    def __init__(
        self,
        binary: Optional[str] = None,
        passphrase: Optional[str] = None,
        keyfile: Optional[str] = None,
    ) -> None:
        if binary is not None:
            self._binary = binary
        else:
            found = shutil.which("authy")
            if found is None:
                raise FileNotFoundError(
                    "authy binary not found on PATH. "
                    "Install authy or pass binary='/path/to/authy'."
                )
            self._binary = found

        self._extra_env: Dict[str, str] = {}
        if passphrase is not None:
            self._extra_env["AUTHY_PASSPHRASE"] = passphrase
        if keyfile is not None:
            self._extra_env["AUTHY_KEYFILE"] = keyfile

    # -- context manager ------------------------------------------------

    def __enter__(self) -> "Authy":
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        # Nothing to clean up — env vars are scoped to subprocess calls,
        # not injected into the current process.
        pass

    # -- internal -------------------------------------------------------

    def _run_cmd(
        self, args: List[str], stdin: Optional[str] = None
    ) -> dict:
        """Run an authy CLI command with ``--json`` and return parsed output.

        Raises a typed :class:`AuthyError` subclass on non-zero exit.
        """
        cmd = [self._binary, "--json"] + args
        env = {**os.environ, **self._extra_env}

        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            input=stdin,
            env=env,
        )

        if result.returncode != 0:
            # Parse structured error from stderr
            try:
                error_body = json.loads(result.stderr)
                error = error_body["error"]
            except (json.JSONDecodeError, KeyError):
                raise AuthyError(
                    exit_code=result.returncode,
                    error_code="unknown",
                    message=result.stderr.strip() or f"authy exited with code {result.returncode}",
                )
            raise _map_error(error, result.returncode)

        if result.stdout.strip():
            return json.loads(result.stdout)
        return {}

    # -- public API -----------------------------------------------------

    def get(self, name: str) -> str:
        """Get a secret value. Raises :class:`SecretNotFound` if missing."""
        result = self._run_cmd(["get", name])
        return result["value"]

    def get_or_none(self, name: str) -> Optional[str]:
        """Get a secret value, returning ``None`` if not found."""
        from .errors import SecretNotFound

        try:
            return self.get(name)
        except SecretNotFound:
            return None

    def store(self, name: str, value: str, force: bool = False) -> None:
        """Store a secret. The value is passed via stdin, never as argv."""
        args = ["store", name]
        if force:
            args.append("--force")
        self._run_cmd(args, stdin=value)

    def remove(self, name: str) -> bool:
        """Remove a secret. Returns ``True`` on success.

        Raises :class:`SecretNotFound` if the secret does not exist.
        """
        self._run_cmd(["remove", name])
        return True

    def rotate(self, name: str, new_value: str) -> int:
        """Rotate a secret to a new value. Returns the new version number.

        The new value is passed via stdin, never as argv.
        Raises :class:`SecretNotFound` if the secret does not exist.
        """
        self._run_cmd(["rotate", name], stdin=new_value)
        # After rotation, fetch the secret to get the new version
        result = self._run_cmd(["get", name])
        return result["version"]

    def list(self, scope: Optional[str] = None) -> List[str]:
        """List secret names, optionally filtered by a policy scope."""
        args = ["list"]
        if scope is not None:
            args.extend(["--scope", scope])
        result = self._run_cmd(args)
        return [s["name"] for s in result.get("secrets", [])]

    def run(
        self,
        command: List[str],
        scope: Optional[str] = None,
    ) -> subprocess.CompletedProcess:
        """Run a subprocess with secrets injected as environment variables.

        Returns the :class:`subprocess.CompletedProcess` from the child.
        Note: this calls authy run which replaces the process, so we invoke
        it as a subprocess and capture its result.
        """
        args = ["run"]
        if scope is not None:
            args.extend(["--scope", scope])
        args.append("--")
        args.extend(command)

        cmd = [self._binary] + args
        env = {**os.environ, **self._extra_env}

        return subprocess.run(cmd, capture_output=True, text=True, env=env)

    def import_dotenv(self, path: str, force: bool = False) -> None:
        """Import secrets from a .env file."""
        args = ["import", path]
        if force:
            args.append("--force")
        self._run_cmd(args)

    def init(self) -> None:
        """Initialize a new vault."""
        self._run_cmd(["init"])

    @staticmethod
    def is_initialized(binary: Optional[str] = None) -> bool:
        """Check whether a vault is initialized. Does not require auth.

        This runs ``authy get __probe`` and checks whether the error is
        *not* ``vault_not_initialized``. A vault-not-initialized error
        (exit 7) means no vault exists; any other error means a vault
        is present (auth may still be required to use it).
        """
        bin_path = binary or shutil.which("authy")
        if bin_path is None:
            raise FileNotFoundError("authy binary not found on PATH.")

        result = subprocess.run(
            [bin_path, "--json", "get", "__probe"],
            capture_output=True,
            text=True,
        )

        if result.returncode == 0:
            return True

        try:
            error_body = json.loads(result.stderr)
            code = error_body["error"]["code"]
        except (json.JSONDecodeError, KeyError):
            # Can't parse — assume initialized (vault exists but errored)
            return True

        return code != "vault_not_initialized"
