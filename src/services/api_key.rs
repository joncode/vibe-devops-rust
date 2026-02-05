//! API key generation, hashing, and validation
use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use rand::RngCore;
use crate::errors::{AppError, AppResult};

const PRODUCT_PREFIX: &str = "vds";
const PREFIX_LEN: usize = 8;
const SECRET_LEN: usize = 32;
const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    pub full_key: String,
    pub prefix: String,
    pub hash: String,
}

pub fn generate_api_key() -> AppResult<GeneratedApiKey> {
    let prefix = gen_str(PREFIX_LEN);
    let secret = gen_str(SECRET_LEN);
    let full_key = format!("{}_{}_{}", PRODUCT_PREFIX, prefix, secret);
    let hash = hash_api_key(&full_key)?;
    Ok(GeneratedApiKey { full_key, prefix, hash })
}

pub fn hash_api_key(key: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(key.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hash failed: {}", e)))
}

pub fn verify_api_key(key: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash: {}", e)))?;
    Ok(Argon2::default().verify_password(key.as_bytes(), &parsed).is_ok())
}

pub fn extract_api_key_prefix(key: &str) -> Option<String> {
    let p: Vec<&str> = key.split('_').collect();
    if p.len() == 3 && p[0] == PRODUCT_PREFIX && p[1].len() == PREFIX_LEN {
        Some(p[1].to_string())
    } else {
        None
    }
}

fn gen_str(len: usize) -> String {
    let mut r = Vec::with_capacity(len);
    let n = CHARS.len();
    let max = 256 - (256 % n);
    let mut rng = rand::thread_rng();
    while r.len() < len {
        let mut b = [0u8; 1];
        rng.fill_bytes(&mut b);
        if (b[0] as usize) < max {
            r.push(CHARS[b[0] as usize % n]);
        }
    }
    String::from_utf8(r).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify() {
        let k = generate_api_key().unwrap();
        assert!(k.full_key.starts_with("vds_"));
        assert!(verify_api_key(&k.full_key, &k.hash).unwrap());
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(extract_api_key_prefix("vds_abcd1234_secret"), Some("abcd1234".to_string()));
        assert_eq!(extract_api_key_prefix("invalid"), None);
    }
}
