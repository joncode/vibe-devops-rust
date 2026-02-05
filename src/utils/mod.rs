//! Utility functions

use rand::Rng;
use sha2::{Sha256, Digest};

/// Generate a secure random token
pub fn generate_token(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a SHA-256 hash of a string
pub fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Validate email format (basic validation)
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim().to_lowercase();
    if email.len() < 5 || email.len() > 255 {
        return false;
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    
    if !domain.contains('.') {
        return false;
    }
    
    true
}

/// Validate password strength
pub fn is_valid_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long");
    }
    if password.len() > 128 {
        return Err("Password must be at most 128 characters long");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("TEST@EXAMPLE.COM"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
    }

    #[test]
    fn test_token_generation() {
        let token = generate_token(32);
        assert_eq!(token.len(), 32);
        
        let token2 = generate_token(32);
        assert_ne!(token, token2);
    }

    #[test]
    fn test_sha256_hash() {
        let hash = sha256_hash("test");
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
        
        // Same input should produce same hash
        assert_eq!(hash, sha256_hash("test"));
        
        // Different input should produce different hash
        assert_ne!(hash, sha256_hash("other"));
    }
}
