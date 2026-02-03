use std::error::Error;

/// Boxed error type that is safe to send between threads.
pub type CryptoError = Box<dyn Error + Send + Sync>;

/// [CryptoKeyPair]
/// 公開鍵暗号（非対称鍵暗号）の鍵ペアを扱うための共通インターフェースです。
pub trait CryptoKeyPair: Send + Sync {
    /// 使用しているアルゴリズムの名称 (例: "RSA", "ED25519") を取得します。
    fn algorithm_name(&self) -> &str;

    /// 公開鍵をバイト列として取得します。
    fn public_key_bytes(&self) -> Vec<u8>;

    /// 秘密鍵をバイト列として取得します。
    fn private_key_bytes(&self) -> Vec<u8>;
}

/// [KeyGenerator]
/// 鍵ペアを新規作成するための機能を提供するトレイトです。
pub trait KeyGenerator {
    /// 乱数に基づいて新しいキーペアを生成し、動的なオブジェクトとして返します。
    fn generate(&self) -> Result<Box<dyn CryptoKeyPair>, CryptoError>;
}

/// [Encryptor]
/// データを暗号化・復号するための基本的な機能を提供します。
pub trait Encryptor {
    /// プレーンテキストを暗号化し、暗号文を返します。
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    
    /// 暗号文を復号し、元のプレーンテキストを返します。
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

/// [Signer]
/// データの署名作成と検証を行うための機能を提供します。
pub trait Signer {
    /// 指定されたデータに対してデジタル署名を作成します。
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError>;
    
    /// データと署名を照合し、正当なものか検証します。
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, CryptoError>;
}

/// [SymmetricCrypto]
/// 共通鍵（対称鍵）を用いて高速にデータを暗号化・復号するためのインターフェースです。
pub trait SymmetricCrypto: Send + Sync {
    /// プレーンテキストを共通鍵で暗号化します。
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// 暗号文を共通鍵で復号します。
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// 現在設定されている共通鍵の生バイト列を取得します。
    fn key_bytes(&self) -> Vec<u8>;
}
