pub mod hcvault;
pub mod onepassword;
pub mod pass;
pub mod sops;

use authy::error::Result;

/// Trait for external secret source adapters.
/// Each adapter fetches secrets from an external store and returns them
/// as (name, value) pairs for the shared import pipeline.
pub trait ImportAdapter {
    fn fetch(&self) -> Result<Vec<(String, String)>>;
}
