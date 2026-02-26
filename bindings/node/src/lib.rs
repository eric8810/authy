use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use authy::api::AuthyClient;
use authy::error::AuthyError;

/// Convert an AuthyError into a napi Error.
/// The error message includes the typed error code in brackets for programmatic use.
fn to_napi_err(e: AuthyError) -> napi::Error {
    let code = e.error_code();
    let msg = e.to_string();
    napi::Error::new(Status::GenericFailure, format!("[{code}] {msg}"))
}

/// Options for creating an Authy client.
#[napi(object)]
pub struct AuthyOptions {
    /// Vault passphrase for authentication.
    pub passphrase: Option<String>,
    /// Path to an age keyfile for authentication.
    pub keyfile: Option<String>,
}

/// Options for storing a secret.
#[napi(object)]
pub struct StoreOptions {
    /// Overwrite existing secret if true.
    pub force: Option<bool>,
}

/// Options for listing secrets.
#[napi(object)]
pub struct ListOptions {
    /// Policy scope to filter secrets by.
    pub scope: Option<String>,
}

/// Native Authy client backed by the Rust vault engine.
///
/// No authy binary on PATH needed — the vault engine is compiled
/// directly into this Node.js module.
#[napi]
pub struct Authy {
    client: AuthyClient,
}

#[napi]
impl Authy {
    /// Create a new Authy client.
    ///
    /// Authenticate with `{ passphrase: "..." }` or `{ keyfile: "/path/to/key" }`.
    #[napi(constructor)]
    pub fn new(opts: AuthyOptions) -> napi::Result<Self> {
        let client = if let Some(ref pass) = opts.passphrase {
            AuthyClient::with_passphrase(pass).map_err(to_napi_err)?
        } else if let Some(ref kf) = opts.keyfile {
            AuthyClient::with_keyfile(kf).map_err(to_napi_err)?
        } else {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "Provide passphrase or keyfile in options",
            ));
        };
        Ok(Self { client })
    }

    /// Retrieve a secret value. Throws if not found.
    #[napi]
    pub fn get(&self, name: String) -> napi::Result<String> {
        self.client.get_or_err(&name).map_err(to_napi_err)
    }

    /// Retrieve a secret value, returning null if not found.
    #[napi(js_name = "getOrNull")]
    pub fn get_or_null(&self, name: String) -> napi::Result<Option<String>> {
        self.client.get(&name).map_err(to_napi_err)
    }

    /// Store a secret. Throws SecretAlreadyExists unless force is set.
    #[napi]
    pub fn store(
        &self,
        name: String,
        value: String,
        opts: Option<StoreOptions>,
    ) -> napi::Result<()> {
        let force = opts.and_then(|o| o.force).unwrap_or(false);
        self.client.store(&name, &value, force).map_err(to_napi_err)
    }

    /// Remove a secret. Returns true if it existed.
    #[napi]
    pub fn remove(&self, name: String) -> napi::Result<bool> {
        self.client.remove(&name).map_err(to_napi_err)
    }

    /// Rotate a secret to a new value. Returns the new version number.
    #[napi]
    pub fn rotate(&self, name: String, new_value: String) -> napi::Result<u32> {
        self.client.rotate(&name, &new_value).map_err(to_napi_err)
    }

    /// List secret names, optionally filtered by a policy scope.
    #[napi]
    pub fn list(&self, opts: Option<ListOptions>) -> napi::Result<Vec<String>> {
        let scope = opts.as_ref().and_then(|o| o.scope.as_deref());
        self.client.list(scope).map_err(to_napi_err)
    }

    /// Build an environment variable map from secrets matching a policy scope.
    #[napi(js_name = "buildEnvMap")]
    pub fn build_env_map(
        &self,
        scope: String,
        uppercase: Option<bool>,
        replace_dash: Option<String>,
    ) -> napi::Result<HashMap<String, String>> {
        let uc = uppercase.unwrap_or(true);
        let rd = replace_dash.and_then(|s| s.chars().next());
        self.client
            .build_env_map(&scope, uc, rd)
            .map_err(to_napi_err)
    }

    /// Test whether a policy allows access to a secret.
    #[napi(js_name = "testPolicy")]
    pub fn test_policy(&self, scope: String, secret_name: String) -> napi::Result<bool> {
        self.client
            .test_policy(&scope, &secret_name)
            .map_err(to_napi_err)
    }

    /// Initialize a new vault.
    #[napi(js_name = "initVault")]
    pub fn init_vault(&self) -> napi::Result<()> {
        self.client.init_vault().map_err(to_napi_err)
    }

    /// Check whether a vault is initialized (static, no auth needed).
    #[napi(js_name = "isInitialized")]
    pub fn is_initialized_check() -> bool {
        AuthyClient::is_initialized()
    }
}
