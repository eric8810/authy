use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::{AuthyError, Result};
use crate::types::*;

type HmacSha256 = Hmac<Sha256>;

const TOKEN_PREFIX: &str = "authy_v1.";
const TOKEN_BYTES: usize = 32;

/// A session record stored in the vault (only the HMAC of the token is stored, not the token itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub scope: String,
    pub token_hmac: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub label: Option<String>,
}

/// Generate a session token and its HMAC.
/// Returns (token_string, hmac_bytes).
pub fn generate_token(hmac_key: &[u8]) -> (String, Vec<u8>) {
    let mut token_bytes = [0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut token_bytes);

    let token_string = format!("{}{}", TOKEN_PREFIX, URL_SAFE_NO_PAD.encode(token_bytes));

    let hmac_bytes = compute_token_hmac(&token_string, hmac_key);

    (token_string, hmac_bytes)
}

/// Compute the HMAC of a token.
fn compute_token_hmac(token: &str, hmac_key: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(hmac_key).expect("HMAC can take key of any size");
    mac.update(token.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// Validate a token against stored session records.
/// Returns the matching session record if valid.
pub fn validate_token<'a>(
    token: &str,
    sessions: &'a [SessionRecord],
    hmac_key: &[u8],
) -> Result<&'a SessionRecord> {
    if !token.starts_with(TOKEN_PREFIX) {
        return Err(AuthyError::InvalidToken);
    }

    let token_hmac = compute_token_hmac(token, hmac_key);

    for session in sessions {
        if session.revoked {
            continue;
        }

        // Constant-time comparison
        if session
            .token_hmac
            .ct_eq(&token_hmac)
            .into()
        {
            // Check expiration
            if Utc::now() > session.expires_at {
                return Err(AuthyError::TokenExpired);
            }
            return Ok(session);
        }
    }

    Err(AuthyError::InvalidToken)
}

/// Parse a duration string like "1h", "30m", "7d".
pub fn parse_ttl(s: &str) -> Result<chrono::Duration> {
    let duration: std::time::Duration =
        humantime::parse_duration(s).map_err(|e| AuthyError::Other(format!("Invalid TTL: {}", e)))?;
    chrono::Duration::from_std(duration)
        .map_err(|e| AuthyError::Other(format!("Duration out of range: {}", e)))
}

/// Generate a short unique session ID.
pub fn generate_session_id() -> String {
    let mut bytes = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
