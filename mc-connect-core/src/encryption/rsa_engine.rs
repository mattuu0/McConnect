use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt, pkcs8::{EncodePublicKey, EncodePrivateKey}};
use rsa::signature::{Signer as RsaSignatureSigner, Verifier as RsaSignatureVerifier, SignatureEncoding};
use rsa::pkcs1v15::{SigningKey, VerifyingKey, Signature};
use rsa::sha2::Sha256;
use rand::rngs::OsRng;
use std::error::Error;
use super::traits::{CryptoKeyPair, KeyGenerator, Encryptor, Signer};

/// [RsaKeyPair]
/// RSA アルゴリズムを使用したキーペアの実装です。
/// データの暗号化・復号（PKCS#1 v1.5）および署名・検証に対応しています。
pub struct RsaKeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl CryptoKeyPair for RsaKeyPair {
    fn algorithm_name(&self) -> &str {
        "RSA"
    }

    /// 公開鍵を DER 形式 (SubjectPublicKeyInfo) で取得します。
    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.to_public_key_der().expect("Failed to encode RSA public key").to_vec()
    }

    /// 秘密鍵を DER 形式 (PKCS#8) で取得します。
    fn private_key_bytes(&self) -> Vec<u8> {
        self.private_key.to_pkcs8_der().expect("Failed to encode RSA private key").to_bytes().to_vec()
    }
}

/// [RsaKeyGenerator]
/// RSA 用のキー生成器です。
pub struct RsaKeyGenerator {
    /// 鍵長 (bit)。デフォルトは 4096 です。
    pub bits: usize,
}

impl Default for RsaKeyGenerator {
    fn default() -> Self {
        Self { bits: 4096 }
    }
}

impl KeyGenerator for RsaKeyGenerator {
    /// 新しい RSA キーペアを生成します。
    fn generate(&self) -> Result<Box<dyn CryptoKeyPair>, Box<dyn Error>> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, self.bits)?;
        let public_key = RsaPublicKey::from(&private_key);
        Ok(Box::new(RsaKeyPair { private_key, public_key }))
    }
}

impl Encryptor for RsaKeyPair {
    /// PKCS#1 v1.5 仕様でデータを暗号化します。
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut rng = OsRng;
        let enc_data = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
        Ok(enc_data)
    }

    /// PKCS#1 v1.5 仕様でデータを復号します。
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let dec_data = self.private_key.decrypt(Pkcs1v15Encrypt, data)?;
        Ok(dec_data)
    }
}

impl Signer for RsaKeyPair {
    /// RSASSA-PKCS1-v1_5 と SHA-256 を使用して署名を生成します。
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let signing_key = SigningKey::<Sha256>::new(self.private_key.clone());
        let signature = signing_key.sign(data);
        Ok(signature.to_vec())
    }

    /// 署名の検証を行います。
    fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> Result<bool, Box<dyn Error>> {
        let verifying_key = VerifyingKey::<Sha256>::new(self.public_key.clone());
        let signature = Signature::try_from(signature_bytes)
            .map_err(|_| "Invalid signature format")?;
        
        Ok(verifying_key.verify(data, &signature).is_ok())
    }
}
