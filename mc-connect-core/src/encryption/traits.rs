use std::error::Error;

/// [CryptoKeyPair]
/// 公開鍵と秘密鍵のペアを保持する抽象的なトレイトです。
pub trait CryptoKeyPair: Send + Sync {
    /// キーペアの種類（"RSA", "ED25519" など）を返します。
    fn algorithm_name(&self) -> &str;

    /// 公開鍵を PEM 形式またはバイト列でシリアライズして取得します。
    fn public_key_bytes(&self) -> Vec<u8>;

    /// 秘密鍵を PEM 形式またはバイト列でシリアライズして取得します。
    fn private_key_bytes(&self) -> Vec<u8>;
}

/// [KeyGenerator]
/// 新しいキーペアを生成するための抽象的なトレイトです。
pub trait KeyGenerator {
    /// 新しいキーペアをランダムに生成します。
    fn generate(&self) -> Result<Box<dyn CryptoKeyPair>, Box<dyn Error>>;
}

/// [Encryptor]
/// データの暗号化と復号を行うためのトレイトです。
/// ※RSAなどの公開鍵暗号で使用されます。
pub trait Encryptor {
    /// データを暗号化します。
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    /// データを復号します。
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

/// [Signer]
/// データの署名と検証を行うためのトレイトです。
pub trait Signer {
    /// データに署名します。
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    /// 署名の正当性を検証します。
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, Box<dyn Error>>;
}
