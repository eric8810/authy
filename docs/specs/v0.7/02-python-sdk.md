# 02 — Python SDK

## Summary

Publish an `authy-secrets` Python package that wraps the `authy` CLI, providing a native Python API for vault operations. Thin subprocess wrapper — no native bindings, no Rust compilation required.

## Motivation

The primary target segment (agent builders) works in Python: LangChain, CrewAI, AutoGen, OpenAI Agents SDK, Anthropic Agent SDK. Shelling out to `authy get` works but feels foreign:

```python
# Today — awkward
import subprocess
result = subprocess.run(["authy", "get", "db-url"], capture_output=True, text=True)
db_url = result.stdout.strip()
```

A native SDK is the expected experience:

```python
# With SDK
from authy_secrets import Authy
client = Authy()
db_url = client.get("db-url")
```

## Design: CLI Wrapper

The SDK shells out to the `authy` binary with `--json` output. This keeps the SDK thin, avoids FFI/PyO3 complexity, and guarantees the SDK always matches the installed authy version.

**Why not PyO3 / native bindings:**
- Requires Rust toolchain to build wheels, or manylinux/macOS/Windows matrix
- Tight coupling to Rust internals — breaks on vault format changes
- `authy` CLI is the stable interface; `--json` output is the API contract
- Subprocess overhead is negligible for secrets operations (not in hot paths)

## API Surface

```python
from authy_secrets import Authy, AuthyError

# Construction — finds `authy` on PATH
client = Authy()

# Or explicit binary path
client = Authy(binary="/usr/local/bin/authy")

# Or with credentials (set as env vars for subprocess)
client = Authy(passphrase="...")
client = Authy(keyfile="/path/to/key")

# Core operations
value: str = client.get("db-url")                    # raises SecretNotFound
value: str | None = client.get_or_none("db-url")     # returns None if missing
client.store("db-url", "postgres://...")              # raises SecretAlreadyExists
client.store("db-url", "postgres://...", force=True)  # overwrite
client.remove("db-url")                               # returns bool
version: int = client.rotate("db-url", "new-value")  # returns new version

# List
names: list[str] = client.list()
names: list[str] = client.list(scope="deploy")

# Run (subprocess injection)
result = client.run(["curl", "https://api.example.com"], scope="deploy")

# Import
client.import_dotenv(".env")
client.import_from("1password", vault="Engineering")

# Vault management
client.init()
Authy.is_initialized()  # static, no auth needed
```

### Error Types

```python
class AuthyError(Exception):
    """Base error with exit_code and error_code from authy CLI."""
    exit_code: int
    error_code: str
    message: str

class SecretNotFound(AuthyError): ...
class SecretAlreadyExists(AuthyError): ...
class AuthFailed(AuthyError): ...
class PolicyDenied(AuthyError): ...
class VaultNotFound(AuthyError): ...
```

Errors are parsed from `--json` stderr output and mapped to typed exceptions.

### Context Manager

```python
# Credentials scoped to a block
with Authy(keyfile="/path/to/key") as client:
    client.get("db-url")
# env vars cleaned up after block
```

## Implementation

### Internal: CLI Execution

```python
class Authy:
    def _run_cmd(self, args: list[str], stdin: str | None = None) -> dict:
        """Run authy CLI with --json, parse result."""
        cmd = [self._binary, "--json"] + args
        env = {**os.environ, **self._extra_env}

        result = subprocess.run(
            cmd, capture_output=True, text=True, env=env
        )

        if result.returncode != 0:
            error = json.loads(result.stderr)["error"]
            raise _map_error(error, result.returncode)

        return json.loads(result.stdout) if result.stdout.strip() else {}
```

Every public method calls `_run_cmd` with the appropriate subcommand and arguments, then extracts the relevant field from the JSON response.

### `get(name)` Example

```python
def get(self, name: str) -> str:
    """Get a secret value. Raises SecretNotFound if missing."""
    result = self._run_cmd(["get", name])
    return result["value"]

def get_or_none(self, name: str) -> str | None:
    """Get a secret value, returning None if not found."""
    try:
        return self.get(name)
    except SecretNotFound:
        return None
```

### `store(name, value)` Example

```python
def store(self, name: str, value: str, force: bool = False) -> None:
    """Store a secret. Value is passed via stdin, never argv."""
    args = ["store", name]
    if force:
        args.append("--force")
    self._run_cmd(args, stdin=value)
```

Note: secret values are passed via stdin to the authy subprocess, matching the CLI convention (never in argv).

## Package Structure

```
packages/python/
  pyproject.toml
  src/
    authy_secrets/
      __init__.py       # re-exports Authy, errors
      client.py         # Authy class
      errors.py         # AuthyError hierarchy
  tests/
    test_client.py      # unit tests (mock subprocess)
    test_integration.py # integration tests (real authy binary)
  README.md
```

### `pyproject.toml`

```toml
[project]
name = "authy-secrets"
version = "0.7.0"
description = "Python SDK for the authy secrets manager"
requires-python = ">=3.9"
license = "MIT"
keywords = ["secrets", "vault", "agents", "ai"]

[project.urls]
Homepage = "https://github.com/eric8810/authy"
Documentation = "https://github.com/eric8810/authy/tree/main/packages/python"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
```

No runtime dependencies. stdlib only (`subprocess`, `json`, `os`).

## Tests

### Unit Tests (mocked subprocess)

- `test_get_returns_value` — mock authy get, assert value returned
- `test_get_not_found_raises` — mock exit code 3, assert SecretNotFound
- `test_store_passes_value_via_stdin` — mock, verify stdin was passed
- `test_list_returns_names` — mock authy list --json, assert list
- `test_auth_failed_raises` — mock exit code 2, assert AuthFailed
- `test_custom_binary_path` — verify binary path used in subprocess call
- `test_credentials_in_env` — verify AUTHY_PASSPHRASE/KEYFILE set in subprocess env

### Integration Tests (require `authy` binary)

- `test_init_store_get_roundtrip` — init vault, store, get, verify value
- `test_list_with_scope` — create policy, store secrets, list with scope filter
- `test_store_duplicate_raises` — store twice, assert SecretAlreadyExists
- `test_rotate_bumps_version` — store, rotate, verify version incremented

Integration tests are marked with `@pytest.mark.integration` and skipped if `authy` is not on PATH.

## Acceptance Criteria

- [ ] `pip install authy-secrets` installs with no native deps
- [ ] `Authy().get("name")` retrieves a secret value
- [ ] `Authy().store("name", "value")` stores via stdin (not argv)
- [ ] `Authy().list()` and `Authy().list(scope=...)` return filtered names
- [ ] Errors map to typed Python exceptions with exit codes
- [ ] `Authy(keyfile=...)` and `Authy(passphrase=...)` pass creds via env
- [ ] Unit tests pass with mocked subprocess (no authy binary needed)
- [ ] Integration tests pass against real authy binary
- [ ] Package published to PyPI as `authy-secrets`
