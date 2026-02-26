use std::collections::HashMap;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::create_exception;

use authy::api::AuthyClient;
use authy::error::AuthyError;

// ── Exception hierarchy ──────────────────────────────────────────

create_exception!(authy_cli, AuthyException, PyException);
create_exception!(authy_cli, SecretNotFound, AuthyException);
create_exception!(authy_cli, SecretAlreadyExists, AuthyException);
create_exception!(authy_cli, AuthFailed, AuthyException);
create_exception!(authy_cli, PolicyNotFound, AuthyException);
create_exception!(authy_cli, AccessDenied, AuthyException);
create_exception!(authy_cli, VaultNotInitialized, AuthyException);

/// Convert an AuthyError into a typed Python exception.
fn to_py_err(e: AuthyError) -> PyErr {
    let msg = e.to_string();
    let code = e.error_code().to_string();
    let exit_code = e.exit_code();

    let py_err = match &e {
        AuthyError::SecretNotFound(_) => SecretNotFound::new_err(msg),
        AuthyError::SecretAlreadyExists(_) => SecretAlreadyExists::new_err(msg),
        AuthyError::AuthFailed(_) | AuthyError::Decryption(_) | AuthyError::InvalidKeyfile(_) => {
            AuthFailed::new_err(msg)
        }
        AuthyError::PolicyNotFound(_) => PolicyNotFound::new_err(msg),
        AuthyError::AccessDenied { .. }
        | AuthyError::TokenReadOnly
        | AuthyError::RunOnly => AccessDenied::new_err(msg),
        AuthyError::VaultNotInitialized => VaultNotInitialized::new_err(msg),
        _ => AuthyException::new_err(msg),
    };

    // Attach code and exit_code attributes via Python
    Python::with_gil(|py| {
        if let Ok(val) = py_err.value(py).extract::<Bound<'_, PyAny>>() {
            let _ = val.setattr("code", code);
            let _ = val.setattr("exit_code", exit_code);
        }
    });

    py_err
}

// ── Python class ─────────────────────────────────────────────────

/// Native Authy client backed by the Rust vault engine.
///
/// No authy binary on PATH needed — the vault engine is compiled
/// directly into this Python module.
#[pyclass(name = "Authy")]
struct PyAuthy {
    client: AuthyClient,
}

#[pymethods]
impl PyAuthy {
    /// Create a new Authy client.
    ///
    /// Authenticate with one of:
    /// - `passphrase="..."` — vault passphrase
    /// - `keyfile="/path/to/key.age"` — age keyfile
    /// - `from_env=True` — read AUTHY_KEYFILE or AUTHY_PASSPHRASE from env
    #[new]
    #[pyo3(signature = (*, passphrase=None, keyfile=None, from_env=false))]
    fn new(
        passphrase: Option<&str>,
        keyfile: Option<&str>,
        from_env: bool,
    ) -> PyResult<Self> {
        let client = if let Some(pass) = passphrase {
            AuthyClient::with_passphrase(pass).map_err(to_py_err)?
        } else if let Some(kf) = keyfile {
            AuthyClient::with_keyfile(kf).map_err(to_py_err)?
        } else if from_env {
            AuthyClient::from_env().map_err(to_py_err)?
        } else {
            return Err(AuthyException::new_err(
                "Provide passphrase=, keyfile=, or from_env=True",
            ));
        };
        Ok(Self { client })
    }

    /// Retrieve a secret value. Raises SecretNotFound if missing.
    fn get(&self, name: &str) -> PyResult<String> {
        self.client
            .get_or_err(name)
            .map_err(to_py_err)
    }

    /// Retrieve a secret value, returning None if not found.
    fn get_or_none(&self, name: &str) -> PyResult<Option<String>> {
        self.client.get(name).map_err(to_py_err)
    }

    /// Store a secret. Raises SecretAlreadyExists unless force=True.
    #[pyo3(signature = (name, value, force=false))]
    fn store(&self, name: &str, value: &str, force: bool) -> PyResult<()> {
        self.client.store(name, value, force).map_err(to_py_err)
    }

    /// Remove a secret. Returns True if it existed.
    fn remove(&self, name: &str) -> PyResult<bool> {
        self.client.remove(name).map_err(to_py_err)
    }

    /// Rotate a secret to a new value. Returns the new version number.
    fn rotate(&self, name: &str, new_value: &str) -> PyResult<u32> {
        self.client.rotate(name, new_value).map_err(to_py_err)
    }

    /// List secret names, optionally filtered by a policy scope.
    #[pyo3(signature = (scope=None))]
    fn list(&self, scope: Option<&str>) -> PyResult<Vec<String>> {
        self.client.list(scope).map_err(to_py_err)
    }

    /// Build an environment variable map from secrets matching a policy scope.
    ///
    /// Args:
    ///     scope: Policy name to filter secrets
    ///     uppercase: Convert names to UPPER_CASE
    ///     replace_dash: Character to replace dashes with (e.g. "_")
    #[pyo3(signature = (scope, uppercase=true, replace_dash=Some('_')))]
    fn build_env_map(
        &self,
        scope: &str,
        uppercase: bool,
        replace_dash: Option<char>,
    ) -> PyResult<HashMap<String, String>> {
        self.client
            .build_env_map(scope, uppercase, replace_dash)
            .map_err(to_py_err)
    }

    /// Test whether a policy allows access to a secret.
    fn test_policy(&self, scope: &str, secret_name: &str) -> PyResult<bool> {
        self.client.test_policy(scope, secret_name).map_err(to_py_err)
    }

    /// Initialize a new vault.
    fn init_vault(&self) -> PyResult<()> {
        self.client.init_vault().map_err(to_py_err)
    }

    /// Check whether a vault is initialized (static, no auth needed).
    #[staticmethod]
    fn is_initialized() -> bool {
        AuthyClient::is_initialized()
    }
}

// ── Module registration ──────────────────────────────────────────

/// Native Authy binding module.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAuthy>()?;
    m.add("AuthyException", m.py().get_type::<AuthyException>())?;
    m.add("SecretNotFound", m.py().get_type::<SecretNotFound>())?;
    m.add("SecretAlreadyExists", m.py().get_type::<SecretAlreadyExists>())?;
    m.add("AuthFailed", m.py().get_type::<AuthFailed>())?;
    m.add("PolicyNotFound", m.py().get_type::<PolicyNotFound>())?;
    m.add("AccessDenied", m.py().get_type::<AccessDenied>())?;
    m.add("VaultNotInitialized", m.py().get_type::<VaultNotInitialized>())?;
    Ok(())
}
