use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use std::fmt;

#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
    pub tag: Vec<u8>,
}

#[derive(Clone)]
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl fmt::Debug for EncryptionService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptionService")
            .field("cipher", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Invalid master key: {0}")]
    InvalidMasterKey(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid encrypted data format")]
    InvalidFormat,
}

impl EncryptionService {
    pub fn new(master_key_hex: &str) -> Result<Self, EncryptionError> {
        let key_bytes = hex::decode(master_key_hex)
            .map_err(|e| EncryptionError::InvalidMasterKey(format!("Invalid hex: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidMasterKey(
                format!("Expected 32 bytes (256 bits), got {} bytes", key_bytes.len())
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        let mut iv = vec![0u8; 12];
        OsRng.fill_bytes(&mut iv);

        let nonce = Nonce::from_slice(&iv);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let tag = ciphertext[ciphertext.len() - 16..].to_vec();
        let ciphertext_only = ciphertext[..ciphertext.len() - 16].to_vec();

        Ok(EncryptedData {
            ciphertext: ciphertext_only,
            iv,
            tag,
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<String, EncryptionError> {
        if encrypted.iv.len() != 12 {
            return Err(EncryptionError::InvalidFormat);
        }

        if encrypted.tag.len() != 16 {
            return Err(EncryptionError::InvalidFormat);
        }

        let nonce = Nonce::from_slice(&encrypted.iv);

        let mut ciphertext_with_tag = encrypted.ciphertext.clone();
        ciphertext_with_tag.extend_from_slice(&encrypted.tag);

        let plaintext_bytes = self.cipher
            .decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }

    #[allow(dead_code)]
    pub fn mask_api_key(api_key: &str) -> String {
        if api_key.len() <= 10 {
            return format!("{}***", &api_key[..2.min(api_key.len())]);
        }

        let prefix_len = 10.min(api_key.len() / 3);
        let suffix_len = 4.min(api_key.len() / 4);

        format!(
            "{}...{}",
            &api_key[..prefix_len],
            &api_key[api_key.len() - suffix_len..]
        )
    }

    pub fn extract_prefix_suffix(api_key: &str) -> (String, String) {
        let prefix_len = 10.min(api_key.len() / 3);
        let suffix_len = 4.min(api_key.len() / 4);

        let prefix = api_key[..prefix_len].to_string();
        let suffix = if api_key.len() >= suffix_len {
            api_key[api_key.len() - suffix_len..].to_string()
        } else {
            String::new()
        };

        (prefix, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_service() -> EncryptionService {
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        EncryptionService::new(test_key).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let service = get_test_service();
        let plaintext = "sk-proj-test1234567890abcdefghijklmnopqrstuvwxyz";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_plaintexts_different_ciphertexts() {
        let service = get_test_service();
        let plaintext1 = "test-key-1";
        let plaintext2 = "test-key-2";

        let encrypted1 = service.encrypt(plaintext1).unwrap();
        let encrypted2 = service.encrypt(plaintext2).unwrap();

        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_same_plaintext_different_ivs() {
        let service = get_test_service();
        let plaintext = "test-key";

        let encrypted1 = service.encrypt(plaintext).unwrap();
        let encrypted2 = service.encrypt(plaintext).unwrap();

        assert_ne!(encrypted1.iv, encrypted2.iv);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        let decrypted1 = service.decrypt(&encrypted1).unwrap();
        let decrypted2 = service.decrypt(&encrypted2).unwrap();

        assert_eq!(plaintext, decrypted1);
        assert_eq!(plaintext, decrypted2);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let service1 = get_test_service();
        let service2 = EncryptionService::new(
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
        ).unwrap();

        let plaintext = "test-key";
        let encrypted = service1.encrypt(plaintext).unwrap();

        let result = service2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let service = get_test_service();
        let plaintext = "test-key";

        let mut encrypted = service.encrypt(plaintext).unwrap();
        encrypted.ciphertext[0] ^= 0x01;

        let result = service.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_tag_fails() {
        let service = get_test_service();
        let plaintext = "test-key";

        let mut encrypted = service.encrypt(plaintext).unwrap();
        encrypted.tag[0] ^= 0x01;

        let result = service.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_master_key_length() {
        let short_key = "0123456789abcdef";
        let result = EncryptionService::new(short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_master_key_hex() {
        let invalid_hex = "not-a-valid-hex-string-zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let result = EncryptionService::new(invalid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_mask_api_key_formats_correctly() {
        let key1 = "sk-proj-abc123def456ghi789jkl012mno345pqr678stu901";
        let masked1 = EncryptionService::mask_api_key(key1);
        assert!(masked1.starts_with("sk-proj-ab"));
        assert!(masked1.ends_with("u901"));
        assert!(masked1.contains("..."));

        let key2 = "short";
        let masked2 = EncryptionService::mask_api_key(key2);
        assert!(masked2.starts_with("sh"));
        assert!(masked2.ends_with("***"));
    }

    #[test]
    fn test_extract_prefix_suffix() {
        let key = "sk-proj-abc123def456ghi789jkl012mno345pqr678stu901";
        let (prefix, suffix) = EncryptionService::extract_prefix_suffix(key);

        assert_eq!(prefix, "sk-proj-ab");
        assert_eq!(suffix, "u901");
    }

    #[test]
    fn test_encrypted_data_format() {
        let service = get_test_service();
        let plaintext = "test-key";
        let encrypted = service.encrypt(plaintext).unwrap();

        assert_eq!(encrypted.iv.len(), 12);
        assert_eq!(encrypted.tag.len(), 16);
        assert!(!encrypted.ciphertext.is_empty());
    }
}
