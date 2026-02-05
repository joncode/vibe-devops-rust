//! API Key generation, hashing, and validation service

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;

use crate::errors::AppResult;
use crate::services::auth::{hash_password, verify_password};

/// API key prefix for service accounts
const API_KEY_PREFIX: &str = "vds"; // vibe-devops-server

/// Length of the random portion of the API key (in bytes, before base64 encoding)
const API_KEY_RANDOM_BYTES: usize = 32;

/// Generated API key with its components
#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    /// The full API key (only available at generation time)
    pub full_key: String,
    /// The prefix portion for quick lookup (first 8 chars after the prefix_)
    pub prefix: String,
    /// The Argon2 hash of the full key for storage
    pub hash: String,
}

/// Generate a new API key
///
/// Format: vds_{base64_random}
/// Example: vds_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6
///
/// The key is designed to be:
/// - URL-safe (using base64url encoding without padding)
/// - Long enough for security (32 random bytes = 256 bits)
/// - Identifiable by prefix
pub fn generate_api_key() -> AppResult<GeneratedApiKey> {
    // Generate cryptographically secure random bytes
    let mut random_bytes = [0u8; API_KEY_RANDOM_BYTES];
    rand::thread_rng().fill_bytes(&mut random_bytes);

    // Encode as URL-safe base64 (no padding)
    let random_part = URL_SAFE_NO_PAD.encode(random_bytes);

    // Build the full key
    let full_key = format!("{}_{}", API_KEY_PREFIX, random_part);

    // Extract prefix (first 8 chars of random part, for lookup)
    let prefix = format!("{}_{}", API_KEY_PREFIX, &random_part[..8]);

    // Hash the full key using Argon2
    let hash = hash_password(&full_key)?;

    Ok(GeneratedApiKey {
        full_key,
        prefix,
        hash,
    })
}

/// Validate an API key format (without checking against database)
///
/// Returns the prefix if valid, None if invalid format
pub fn validate_api_key_format(api_key: &str) -> Option<String> {
    // Check prefix
    if !api_key.starts_with(&format!("{}_", API_KEY_PREFIX)) {
        return None;
    }

    // Extract the random part
    let random_part = &api_key[(API_KEY_PREFIX.len() + 1)..];

    // Check minimum length (base64 of 32 bytes = 43 chars)
    if random_part.len() < 40 {
        return None;
    }

    // Check it's valid base64url
    if URL_SAFE_NO_PAD.decode(random_part).is_err() {
        return None;
    }

    // Return the prefix for lookup
    Some(format!("{}_{}", API_KEY_PREFIX, &random_part[..8]))
}

/// Extract the lookup prefix from an API key
///
/// Used to find potential matching service accounts in the database
pub fn extract_api_key_prefix(api_key: &str) -> Option<String> {
    validate_api_key_format(api_key)
}

/// Verify an API key against its hash
///
/// Uses constant-time comparison via Argon2
pub fn verify_api_key(api_key: &str, hash: &str) -> AppResult<bool> {
    verify_password(api_key, hash)
}

/// Parse a Bearer token from an Authorization header
///
/// Expects format: "Bearer <api_key>"
pub fn parse_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key().unwrap();

        // Check format
        assert!(key.full_key.starts_with("vds_"));
        assert!(key.prefix.starts_with("vds_"));

        // Prefix should be shorter than full key
        assert!(key.prefix.len() < key.full_key.len());

        // Hash should be Argon2 format
        assert!(key.hash.starts_with("$argon2"));
    }

    #[test]
    fn test_validate_api_key_format() {
        let key = generate_api_key().unwrap();

        // Valid key should return prefix
        let prefix = validate_api_key_format(&key.full_key);
        assert!(prefix.is_some());
        assert_eq!(prefix.unwrap(), key.prefix);

        // Invalid keys
        assert!(validate_api_key_format("invalid").is_none());
        assert!(validate_api_key_format("vds_short").is_none());
        assert!(
            validate_api_key_format("wrong_prefix_A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8").is_none()
        );
    }

    #[test]
    fn test_verify_api_key() {
        let key = generate_api_key().unwrap();

        // Correct key should verify
        assert!(verify_api_key(&key.full_key, &key.hash).unwrap());

        // Wrong key should not verify
        let other_key = generate_api_key().unwrap();
        assert!(!verify_api_key(&other_key.full_key, &key.hash).unwrap());

        // Modified key should not verify
        let modified = format!("{}x", &key.full_key[..key.full_key.len() - 1]);
        assert!(!verify_api_key(&modified, &key.hash).unwrap());
    }

    #[test]
    fn test_uniqueness() {
        // Generate many keys and ensure they're unique
        let mut keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        for _ in 0..100 {
            let key = generate_api_key().unwrap();
            assert!(keys.insert(key.full_key), "Duplicate key generated");
        }
    }

    #[test]
    fn test_parse_bearer_token() {
        assert_eq!(parse_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(parse_bearer_token("Bearer "), Some(""));
        assert_eq!(parse_bearer_token("Basic abc123"), None);
        assert_eq!(parse_bearer_token("abc123"), None);
    }
}
