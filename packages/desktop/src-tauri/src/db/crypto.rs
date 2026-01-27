// db/crypto.rs - Encryption utilities for sensitive data at rest
//
// Uses AES-256-GCM for authenticated encryption of sensitive fields
// stored in the SQLite database (OAuth tokens, API keys, etc.)

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

use crate::{Error, Result};

/// Encrypted data prefix to identify encrypted values
const ENCRYPTED_PREFIX: &str = "enc:v1:";

/// Static encryption key derived at startup
static ENCRYPTION_KEY: OnceLock<[u8; 32]> = OnceLock::new();

/// Initialize the encryption key
///
/// Key derivation priority:
/// 1. FOCUSFLOW_ENCRYPTION_KEY environment variable (for advanced users)
/// 2. Machine-specific key derived from hostname + username + static salt
///
/// This should be called once at application startup.
pub fn init_encryption() -> Result<()> {
    ENCRYPTION_KEY.get_or_init(|| derive_encryption_key());
    Ok(())
}

/// Derive encryption key from machine-specific information
///
/// Combines hostname, username, and a static application salt to create
/// a unique key per machine. This isn't perfect security, but prevents
/// casual database file copying from exposing plaintext secrets.
fn derive_encryption_key() -> [u8; 32] {
    // Check for environment variable override first
    if let Ok(key_str) = std::env::var("FOCUSFLOW_ENCRYPTION_KEY") {
        if key_str.len() >= 32 {
            let mut hasher = Sha256::new();
            hasher.update(key_str.as_bytes());
            return hasher.finalize().into();
        }
        tracing::warn!(
            "FOCUSFLOW_ENCRYPTION_KEY is set but too short (need 32+ chars), using machine key"
        );
    }

    // Derive from machine-specific info
    let mut hasher = Sha256::new();

    // Static application salt (not secret, but adds uniqueness)
    hasher.update(b"FocusFlow-Desktop-v1-EncryptionKey-2024");

    // Machine hostname
    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    // Username
    if let Ok(username) = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .or_else(|_| std::env::var("LOGNAME"))
    {
        hasher.update(username.as_bytes());
    }

    // Home directory path (adds more machine specificity)
    if let Some(home) = dirs::home_dir() {
        hasher.update(home.to_string_lossy().as_bytes());
    }

    hasher.finalize().into()
}

/// Get the encryption key, initializing if needed
fn get_key() -> &'static [u8; 32] {
    ENCRYPTION_KEY.get_or_init(|| derive_encryption_key())
}

/// Encrypt a plaintext string
///
/// Returns a hex-encoded string with format: "enc:v1:<nonce_hex>:<ciphertext_hex>"
/// The "enc:v1:" prefix allows detecting encrypted vs plaintext values.
pub fn encrypt(plaintext: &str) -> Result<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let key = get_key();
    let cipher = Aes256Gcm::new(key.into());

    // Generate a random 12-byte nonce
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| Error::Crypto(format!("Encryption failed: {}", e)))?;

    // Format: enc:v1:<nonce_hex>:<ciphertext_hex>
    Ok(format!(
        "{}{}:{}",
        ENCRYPTED_PREFIX,
        hex::encode(nonce_bytes),
        hex::encode(ciphertext)
    ))
}

/// Decrypt an encrypted string
///
/// Accepts both encrypted (prefixed) and plaintext strings for backward compatibility.
/// Plaintext strings are returned as-is, enabling transparent migration.
pub fn decrypt(encrypted: &str) -> Result<String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    // Check if this is actually encrypted
    if !encrypted.starts_with(ENCRYPTED_PREFIX) {
        // Return plaintext as-is (backward compatibility)
        tracing::debug!("Value is not encrypted, returning as-is");
        return Ok(encrypted.to_string());
    }

    // Parse the encrypted format
    let without_prefix = &encrypted[ENCRYPTED_PREFIX.len()..];
    let parts: Vec<&str> = without_prefix.split(':').collect();

    if parts.len() != 2 {
        return Err(Error::Crypto(
            "Invalid encrypted format: expected nonce:ciphertext".to_string(),
        ));
    }

    let nonce_bytes = hex::decode(parts[0])
        .map_err(|e| Error::Crypto(format!("Invalid nonce hex: {}", e)))?;

    if nonce_bytes.len() != 12 {
        return Err(Error::Crypto(format!(
            "Invalid nonce length: expected 12, got {}",
            nonce_bytes.len()
        )));
    }

    let ciphertext = hex::decode(parts[1])
        .map_err(|e| Error::Crypto(format!("Invalid ciphertext hex: {}", e)))?;

    let key = get_key();
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| Error::Crypto(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext_bytes)
        .map_err(|e| Error::Crypto(format!("Invalid UTF-8 in decrypted data: {}", e)))
}

/// Check if a value is encrypted
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTED_PREFIX)
}

/// Encrypt if not already encrypted (for migration)
pub fn encrypt_if_needed(value: &str) -> Result<String> {
    if value.is_empty() || is_encrypted(value) {
        Ok(value.to_string())
    } else {
        encrypt(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "my-secret-api-key-12345";
        let encrypted = encrypt(plaintext).unwrap();

        assert!(encrypted.starts_with(ENCRYPTED_PREFIX));
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_plaintext_passthrough() {
        let plaintext = "not-encrypted-value";
        let result = decrypt(plaintext).unwrap();
        assert_eq!(result, plaintext);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(encrypt("").unwrap(), "");
        assert_eq!(decrypt("").unwrap(), "");
    }

    #[test]
    fn test_is_encrypted() {
        assert!(!is_encrypted("plaintext"));
        assert!(!is_encrypted(""));
        assert!(is_encrypted("enc:v1:aabbccdd:112233"));
    }

    #[test]
    fn test_encrypt_if_needed() {
        let plaintext = "secret";
        let encrypted = encrypt(plaintext).unwrap();

        // Plaintext should be encrypted
        let result1 = encrypt_if_needed(plaintext).unwrap();
        assert!(is_encrypted(&result1));

        // Already encrypted should pass through
        let result2 = encrypt_if_needed(&encrypted).unwrap();
        assert_eq!(result2, encrypted);
    }

    #[test]
    fn test_different_encryptions_differ() {
        // Same plaintext should produce different ciphertexts (due to random nonce)
        let plaintext = "my-secret";
        let encrypted1 = encrypt(plaintext).unwrap();
        let encrypted2 = encrypt(plaintext).unwrap();

        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same value
        assert_eq!(decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_unicode_support() {
        let plaintext = "API key with unicode: \u{1F512} \u{1F511}";
        let encrypted = encrypt(plaintext).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
