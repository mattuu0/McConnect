use aes_gcm::{
    aead::{Aead, KeyInit, AeadCore},
    Aes256Gcm, Key, Nonce
};
use rand::RngCore;
use rand::rngs::OsRng;
use super::traits::{SymmetricCrypto, CryptoError};

pub struct AesGcmEngine {
    cipher: Aes256Gcm,
    key: Vec<u8>,
}

impl AesGcmEngine {
    pub fn new_random() -> Self {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        Self {
            cipher,
            key: key_bytes.to_vec(),
        }
    }

    pub fn from_key(key_bytes: &[u8]) -> Result<Self, CryptoError> {
        if key_bytes.len() != 32 {
            return Err("AES-256 の鍵は正確に 32バイトである必要があります。".into());
        }
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        Ok(Self {
            cipher,
            key: key_bytes.to_vec(),
        })
    }
}

impl SymmetricCrypto for AesGcmEngine {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)
            .map_err(|e| format!("AES-GCM 暗号化に失敗しました: {}", e))?;

        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < 12 {
            return Err("暗号文が短すぎます。Nonce (12バイト) が含まれていません。".into());
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("AES-GCM 復号に失敗しました: {} (データが改ざんされている可能性があります)", e))?;
        Ok(plaintext)
    }

    fn key_bytes(&self) -> Vec<u8> {
        self.key.clone()
    }
}
