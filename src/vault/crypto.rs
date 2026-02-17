use std::io::{Read, Write};

use age::secrecy::ExposeSecret;
use hkdf::Hkdf;
use sha2::Sha256;

use crate::error::{AuthyError, Result};

/// Encrypt data using a passphrase via age.
pub fn encrypt_with_passphrase(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(
        age::secrecy::Secret::new(passphrase.to_string()),
    );

    let mut encrypted = vec![];
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(|e| AuthyError::Encryption(e.to_string()))?;
    writer
        .write_all(plaintext)
        .map_err(|e| AuthyError::Encryption(e.to_string()))?;
    writer
        .finish()
        .map_err(|e| AuthyError::Encryption(e.to_string()))?;

    Ok(encrypted)
}

/// Decrypt data using a passphrase via age.
pub fn decrypt_with_passphrase(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
    let decryptor = match age::Decryptor::new(ciphertext)
        .map_err(|e| AuthyError::Decryption(e.to_string()))?
    {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err(AuthyError::Decryption("Expected passphrase-encrypted data".into())),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt(
            &age::secrecy::Secret::new(passphrase.to_string()),
            None,
        )
        .map_err(|e| AuthyError::Decryption(e.to_string()))?;
    reader
        .read_to_end(&mut decrypted)
        .map_err(|e| AuthyError::Decryption(e.to_string()))?;

    Ok(decrypted)
}

/// Encrypt data using an age identity (keyfile).
pub fn encrypt_with_keyfile(plaintext: &[u8], pubkey: &str) -> Result<Vec<u8>> {
    let recipient: age::x25519::Recipient = pubkey
        .parse()
        .map_err(|e: &str| AuthyError::Encryption(e.to_string()))?;

    let encryptor = age::Encryptor::with_recipients(vec![Box::new(recipient)])
        .expect("recipients not empty");

    let mut encrypted = vec![];
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(|e| AuthyError::Encryption(e.to_string()))?;
    writer
        .write_all(plaintext)
        .map_err(|e| AuthyError::Encryption(e.to_string()))?;
    writer
        .finish()
        .map_err(|e| AuthyError::Encryption(e.to_string()))?;

    Ok(encrypted)
}

/// Decrypt data using an age identity (keyfile).
pub fn decrypt_with_keyfile(ciphertext: &[u8], identity_str: &str) -> Result<Vec<u8>> {
    let identity: age::x25519::Identity = identity_str
        .parse()
        .map_err(|e: &str| AuthyError::InvalidKeyfile(e.to_string()))?;

    let decryptor = match age::Decryptor::new(ciphertext)
        .map_err(|e| AuthyError::Decryption(e.to_string()))?
    {
        age::Decryptor::Recipients(d) => d,
        _ => return Err(AuthyError::Decryption("Expected recipients-encrypted data".into())),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|e| AuthyError::Decryption(e.to_string()))?;
    reader
        .read_to_end(&mut decrypted)
        .map_err(|e| AuthyError::Decryption(e.to_string()))?;

    Ok(decrypted)
}

/// Derive a sub-key using HKDF-SHA256.
pub fn derive_key(master: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    let hk = Hkdf::<Sha256>::new(None, master);
    let mut okm = vec![0u8; output_len];
    hk.expand(info, &mut okm)
        .expect("HKDF output length too large");
    okm
}

/// Generate a new age keypair. Returns (secret_key_string, public_key_string).
pub fn generate_keypair() -> (String, String) {
    let identity = age::x25519::Identity::generate();
    let secret_key = identity.to_string();
    let public_key = identity.to_public().to_string();
    (secret_key.expose_secret().clone(), public_key)
}
