// oauth/mod.rs - OAuth integration module for calendar providers

pub mod google;
pub mod microsoft;
pub mod provider;
pub mod token;

pub use token::TokenManager;

use crate::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};

/// Generate a cryptographically secure random state parameter for OAuth CSRF protection
pub fn generate_state() -> String {
    use rand::Rng;
    let random_bytes: [u8; 32] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(random_bytes)
}

/// PKCE (Proof Key for Code Exchange) utilities for OAuth2 flow
pub struct Pkce {
    pub code_verifier: String,
    pub code_challenge: String,
}

impl Pkce {
    /// Generate a new PKCE code verifier and challenge pair
    ///
    /// Uses S256 method (SHA256 hash of verifier, base64url encoded)
    pub fn generate() -> Result<Self> {
        use rand::Rng;

        // Generate code_verifier: 43-128 characters from [A-Z, a-z, 0-9, -, ., _, ~]
        let random_bytes: [u8; 32] = rand::thread_rng().gen();
        let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes);

        // Generate code_challenge: BASE64URL(SHA256(code_verifier))
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(hash);

        Ok(Self {
            code_verifier,
            code_challenge,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = Pkce::generate().unwrap();
        assert!(!pkce.code_verifier.is_empty());
        assert!(!pkce.code_challenge.is_empty());
        assert_ne!(pkce.code_verifier, pkce.code_challenge);
    }

    #[test]
    fn test_state_generation() {
        let state1 = generate_state();
        let state2 = generate_state();
        assert!(!state1.is_empty());
        assert!(!state2.is_empty());
        assert_ne!(state1, state2);
    }
}
