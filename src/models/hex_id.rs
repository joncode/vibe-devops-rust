//! Hex ID generation for public-facing, URL-safe identifiers
//!
//! Each model defines its own prefix constant, and this module provides
//! the secure random string generation. No case statements here - the
//! prefix is a characteristic of the model, not the generator.
//!
//! Format: {prefix}_{10 random chars}
//! Safe charset: a-k, m-z, 0, 2-9 (34 chars total, avoids l/1 confusion)
//!
//! Based on: https://gist.github.com/joncode/a39791fdcdd7c19a695e3e6e7af16d7e

use rand::RngCore;

/// Safe character set: a-k, m-z, 0, 2-9 (34 characters)
/// Excludes 'l' and '1' to avoid visual confusion
const SAFE_CHARS: &[u8] = b"abcdefghijkmnopqrstuvwxyz023456789";

/// Maximum retries for collision handling
pub const MAX_RETRIES: u32 = 3;

/// Default length of the random portion of hex_id
pub const DEFAULT_HEX_ID_LENGTH: usize = 10;

/// Trait for models that have a hex_id
/// 
/// Each model implements this trait to define its prefix.
/// The prefix is a characteristic of the model, not the hex_id generator.
/// 
/// # Example
/// ```
/// impl HexId for User {
///     const PREFIX: &'static str = "usr";
/// }
/// ```
pub trait HexId {
    /// The prefix for this model's hex_id (e.g., "usr", "stk", "sid")
    const PREFIX: &'static str;
    
    /// Generate a new hex_id for this model
    fn generate_hex_id() -> String {
        build_hex_id(Self::PREFIX)
    }
    
    /// Generate hex_id with custom length (for testing or special cases)
    fn generate_hex_id_with_length(length: usize) -> String {
        build_hex_id_with_length(Self::PREFIX, length)
    }
}

/// Build a hex_id with the given prefix and default length
pub fn build_hex_id(prefix: &str) -> String {
    build_hex_id_with_length(prefix, DEFAULT_HEX_ID_LENGTH)
}

/// Build a hex_id with the given prefix and custom length
pub fn build_hex_id_with_length(prefix: &str, length: usize) -> String {
    format!("{}_{}", prefix, secure_random_safe_string(length))
}

/// Generate a cryptographically secure random string using the safe charset
/// 
/// Uses rejection sampling to ensure uniform distribution:
/// - Generate random bytes
/// - Reject bytes that would cause modulo bias
/// - Map remaining bytes to safe characters
fn secure_random_safe_string(length: usize) -> String {
    let mut result = Vec::with_capacity(length);
    let n = SAFE_CHARS.len();
    
    // Calculate the maximum byte value that gives uniform distribution
    // 256 % 34 = 18, so we reject bytes >= 238 (256 - 18)
    let max_valid = 256 - (256 % n);
    
    let mut rng = rand::thread_rng();
    
    while result.len() < length {
        let mut byte = [0u8; 1];
        rng.fill_bytes(&mut byte);
        let b = byte[0] as usize;
        
        // Rejection sampling: skip bytes that would cause bias
        if b < max_valid {
            result.push(SAFE_CHARS[b % n]);
        }
    }
    
    String::from_utf8(result).expect("SAFE_CHARS contains only ASCII")
}

/// Validate a hex_id format
/// 
/// Returns true if the hex_id matches the expected format:
/// - Starts with a valid prefix
/// - Followed by underscore
/// - Followed by characters from the safe charset
pub fn validate_hex_id(hex_id: &str, expected_prefix: &str) -> bool {
    // Check prefix
    if !hex_id.starts_with(expected_prefix) {
        return false;
    }
    
    // Check underscore separator
    let after_prefix = &hex_id[expected_prefix.len()..];
    if !after_prefix.starts_with('_') {
        return false;
    }
    
    // Check random portion contains only safe characters
    let random_part = &after_prefix[1..];
    if random_part.is_empty() {
        return false;
    }
    
    random_part.bytes().all(|b| SAFE_CHARS.contains(&b))
}

/// Parse a hex_id and return its components (prefix, random_part)
pub fn parse_hex_id(hex_id: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = hex_id.splitn(2, '_').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestModel;
    impl HexId for TestModel {
        const PREFIX: &'static str = "tst";
    }
    
    #[test]
    fn test_generate_hex_id() {
        let hex_id = TestModel::generate_hex_id();
        assert!(hex_id.starts_with("tst_"));
        assert_eq!(hex_id.len(), 14); // "tst_" (4) + 10 random chars
    }
    
    #[test]
    fn test_hex_id_format() {
        let hex_id = TestModel::generate_hex_id();
        assert!(validate_hex_id(&hex_id, "tst"));
    }
    
    #[test]
    fn test_safe_chars_only() {
        for _ in 0..100 {
            let random = secure_random_safe_string(20);
            for c in random.chars() {
                assert!(
                    ('a'..='k').contains(&c) || 
                    ('m'..='z').contains(&c) || 
                    c == '0' ||
                    ('2'..='9').contains(&c),
                    "Invalid character: {}", c
                );
            }
        }
    }
    
    #[test]
    fn test_no_l_or_1() {
        // Generate many strings and ensure no l or 1
        for _ in 0..1000 {
            let random = secure_random_safe_string(100);
            assert!(!random.contains('l'), "Found 'l' in: {}", random);
            assert!(!random.contains('1'), "Found '1' in: {}", random);
        }
    }
    
    #[test]
    fn test_uniqueness() {
        // Generate many IDs and check for uniqueness
        let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for _ in 0..10000 {
            let id = TestModel::generate_hex_id();
            assert!(ids.insert(id.clone()), "Duplicate ID generated: {}", id);
        }
    }
    
    #[test]
    fn test_parse_hex_id() {
        let hex_id = "usr_abc123def4";
        let (prefix, random) = parse_hex_id(hex_id).unwrap();
        assert_eq!(prefix, "usr");
        assert_eq!(random, "abc123def4");
    }
    
    #[test]
    fn test_validate_hex_id() {
        assert!(validate_hex_id("usr_abcdefghij", "usr"));
        assert!(validate_hex_id("stk_0234567890", "stk"));
        assert!(!validate_hex_id("usr_abcdefghij", "stk")); // wrong prefix
        assert!(!validate_hex_id("usrabcdefghij", "usr"));  // missing underscore
        assert!(!validate_hex_id("usr_", "usr"));           // empty random part
        assert!(!validate_hex_id("usr_abc1def", "usr"));    // contains '1'
        assert!(!validate_hex_id("usr_abcldef", "usr"));    // contains 'l'
    }
}
