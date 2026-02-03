use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt, pkcs8::{EncodePublicKey, EncodePrivateKey, DecodePublicKey, DecodePrivateKey}};
use rsa::signature::{Signer as RsaSignatureSigner, Verifier as RsaSignatureVerifier, SignatureEncoding};
use rsa::pkcs1v15::{SigningKey, VerifyingKey, Signature};
use rsa::sha2::Sha256;
use rand::rngs::OsRng;
use super::traits::{CryptoKeyPair, KeyGenerator, Encryptor, Signer, CryptoError};

pub struct RsaKeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl RsaKeyPair {
    pub fn from_private_der(der: &[u8]) -> Result<Self, CryptoError> {
        let private_key = RsaPrivateKey::from_pkcs8_der(der).map_err(|e| Box::new(e) as CryptoError)?;
        let public_key = RsaPublicKey::from(&private_key);
        Ok(Self { private_key, public_key })
    }

    pub fn from_public_der(der: &[u8]) -> Result<Self, CryptoError> {
        let public_key = RsaPublicKey::from_public_key_der(der).map_err(|e| Box::new(e) as CryptoError)?;
        let dummy_priv = RsaPrivateKey::new(&mut OsRng, 512).map_err(|e| Box::new(e) as CryptoError)?; 
        Ok(Self { private_key: dummy_priv, public_key })
    }
}

impl CryptoKeyPair for RsaKeyPair {
    fn algorithm_name(&self) -> &str {
        "RSA"
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.to_public_key_der().expect("RSA公開鍵のエンコードに失敗しました").to_vec()
    }

    fn private_key_bytes(&self) -> Vec<u8> {
        self.private_key.to_pkcs8_der().expect("RSA秘密鍵のエンコードに失敗しました").to_bytes().to_vec()
    }
}

pub struct RsaKeyGenerator {
    pub bits: usize,
}

impl Default for RsaKeyGenerator {
    fn default() -> Self {
        Self { bits: 4096 }
    }
}

impl KeyGenerator for RsaKeyGenerator {
    fn generate(&self) -> Result<Box<dyn CryptoKeyPair>, CryptoError> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, self.bits).map_err(|e| Box::new(e) as CryptoError)?;
        let public_key = RsaPublicKey::from(&private_key);
        Ok(Box::new(RsaKeyPair { private_key, public_key }))
    }
}

impl Encryptor for RsaKeyPair {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut rng = OsRng;
        let enc_data = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data).map_err(|e| Box::new(e) as CryptoError)?;
        Ok(enc_data)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let dec_data = self.private_key.decrypt(Pkcs1v15Encrypt, data).map_err(|e| Box::new(e) as CryptoError)?;
        Ok(dec_data)
    }
}

impl Signer for RsaKeyPair {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let signing_key = SigningKey::<Sha256>::new(self.private_key.clone());
        let signature = signing_key.sign(data);
        Ok(signature.to_vec())
    }

    fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> Result<bool, CryptoError> {
        let verifying_key = VerifyingKey::<Sha256>::new(self.public_key.clone());
        let signature = Signature::try_from(signature_bytes)
            .map_err(|_| "署名のフォーマットが不正です。")?;
        
        Ok(verifying_key.verify(data, &signature).is_ok())
    }
}
