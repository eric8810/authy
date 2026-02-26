# 02 — Python Native Binding (PyO3)

## Summary

Ship a native Python binding for Authy using PyO3 + maturin. The Rust vault engine compiles directly into the Python package — no separate `authy` binary needed. Published as `authy-cli` on PyPI (replacing the subprocess wrapper `authy-secrets`).

## Motivation

The v0.7.0 Python SDK (`authy-secrets`) shells out to the `authy` CLI binary. This works but has drawbacks:

- Requires `authy` binary on PATH (separate installation step)
- Subprocess overhead per operation
- Error messages are string-parsed from JSON stderr
- No single-binary distribution

A native binding embeds the Rust vault engine into the Python wheel. One `pip install` gets everything.

## Current Behavior

```python
# v0.7.0 — subprocess wrapper
from authy_secrets import Authy
client = Authy()  # requires authy binary on PATH
value = client.get("db-url")  # shells out to: authy --json get db-url
```

## Proposed Behavior

```python
# v0.7.1 — native binding
from authy_cli import Authy
client = Authy(passphrase="...")  # no binary needed
value = client.get("db-url")  # direct Rust FFI call
```

## API Surface

```python
from authy_cli import Authy, AuthyError, SecretNotFound, AuthFailed

# Construction
client = Authy(passphrase="my-passphrase")
client = Authy(keyfile="/path/to/key.age")
client = Authy(from_env=True)  # reads AUTHY_KEYFILE / AUTHY_PASSPHRASE

# Core operations
value: str = client.get("db-url")                    # raises SecretNotFound
value: str | None = client.get_or_none("db-url")     # returns None if missing
client.store("db-url", "postgres://...", force=False)
removed: bool = client.remove("db-url")
version: int = client.rotate("db-url", "new-value")

# List
names: list[str] = client.list()
names: list[str] = client.list(scope="deploy")

# Env map (for subprocess injection)
env: dict[str, str] = client.build_env_map("deploy", uppercase=True, replace_dash="_")

# Policy
allowed: bool = client.test_policy("deploy", "db-url")

# Vault management
client.init_vault()
Authy.is_initialized()  # static, no auth needed
```

### Error Types

```python
class AuthyError(Exception):
    """Base exception with .code (str) and .exit_code (int)."""
    code: str
    exit_code: int

class SecretNotFound(AuthyError): ...       # exit_code=3
class SecretAlreadyExists(AuthyError): ...  # exit_code=5
class AuthFailed(AuthyError): ...           # exit_code=2
class PolicyNotFound(AuthyError): ...       # exit_code=3
class AccessDenied(AuthyError): ...         # exit_code=4
class VaultNotInitialized(AuthyError): ...  # exit_code=7
```

## Implementation

### Directory Structure

```
bindings/python/
├── Cargo.toml              # PyO3 cdylib crate
├── pyproject.toml           # maturin build backend
├── src/lib.rs               # #[pyclass] wrapper around AuthyClient
├── python/authy_cli/
│   ├── __init__.py          # re-exports from native module
│   └── __init__.pyi         # type stubs for IDE support
└── tests/test_binding.py    # pytest tests
```

### Crate: `bindings/python/Cargo.toml`

- PyO3 cdylib with `extension-module` feature
- Depends on `authy` via path `../..` with `default-features = false` (no CLI deps)

### Native Module: `bindings/python/src/lib.rs`

- `#[pyclass] struct PyAuthyClient` wrapping `authy::api::AuthyClient`
- `#[pymethods]` for each API method
- Custom exception types registered via `pyo3::create_exception!`
- Error conversion from `AuthyError` variants to Python exception subclasses

### FFI Note

Returned strings cross the FFI boundary as Python strings (not zeroized on Python side — documented limitation).

## Package Structure

### `pyproject.toml`

```toml
[project]
name = "authy-cli"
version = "0.7.1"
description = "Native Python binding for the authy secrets manager (Rust-powered)"
requires-python = ">=3.9"
license = "MIT"

[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
features = []
module-name = "authy_cli._native"
```

## Tests

### `tests/test_binding.py`

- `test_init_and_store` — create vault, store secret, verify get returns value
- `test_get_not_found` — verify SecretNotFound exception raised
- `test_list_with_scope` — create policy, verify filtered list
- `test_build_env_map` — verify env var map with transformations
- `test_auth_failed` — verify AuthFailed on wrong passphrase
- `test_is_initialized` — static check before/after init
- `test_remove_and_rotate` — full lifecycle

All tests use isolated HOME directories (tempfile) to avoid vault collisions.

## Acceptance Criteria

- [ ] `pip install authy-cli` installs native Rust-powered binding (no authy binary needed)
- [ ] `from authy_cli import Authy` works
- [ ] All API methods (get, store, remove, rotate, list, build_env_map, test_policy, init_vault) work
- [ ] Typed Python exceptions with `.code` and `.exit_code` attributes
- [ ] Type stubs (`.pyi`) provide IDE autocomplete
- [ ] `maturin develop` builds and imports successfully
- [ ] Tests pass with isolated HOME directories
- [ ] Deprecation notice added to `authy-secrets` on PyPI
