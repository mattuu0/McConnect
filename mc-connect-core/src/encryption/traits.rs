use std::error::Error;

/// [CryptoKeyPair]
/// 公開鍵暗号（非対称鍵暗号）の鍵ペアを扱うための共通インターフェースです。
/// RSA や将来的な楕円曲線暗号 (Ed25519 等) を抽象化します。
pub trait CryptoKeyPair: Send + Sync {
    /// 使用しているアルゴリズムの名称 (例: "RSA", "ED25519") を取得します。
    fn algorithm_name(&self) -> &str;

    /// 公開鍵をバイト列として取得します。
    /// ネットワーク経由で相手に送る際に使用します。
    fn public_key_bytes(&self) -> Vec<u8>;

    /// 秘密鍵をバイト列として取得します。
    /// 自身のローカルストレージに保存する際などに使用します。
    fn private_key_bytes(&self) -> Vec<u8>;
}

/// [KeyGenerator]
/// 鍵ペアを新規作成するための機能を提供するトレイトです。
pub trait KeyGenerator {
    /// 乱数に基づいて新しいキーペアを生成し、動的なオブジェクトとして返します。
    fn generate(&self) -> Result<Box<dyn CryptoKeyPair>, Box<dyn Error>>;
}

/// [Encryptor]
/// データを暗号化・復号するための基本的な機能を提供します。
/// 主に公開鍵暗号による小規模なデータ（共通鍵の断片など）の保護に使用します。
pub trait Encryptor {
    /// プレーンテキストを暗号化し、暗号文を返します。
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    
    /// 暗号文を復号し、元のプレーンテキストを返します Lights。
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

/// [Signer]
/// データの署名作成と検証を行うための機能を提供します。
/// 通信相手が本人であることを確認するために使用します。
pub trait Signer {
    /// 指定されたデータに対してデジタル署名を作成します。
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    
    /// データと署名を照合し、正当なものか検証します。
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, Box<dyn Error>>;
}

/// [SymmetricCrypto]
/// 共通鍵（対称鍵）を用いて高速にデータを暗号化・復号するためのインターフェースです。
/// 大容量のメインデータの転送には、このプロトコルを実装した AES 等が使用されます。
pub trait SymmetricCrypto: Send + Sync {
    /// プレーンテキストを共通鍵で暗号化します。
    /// 認証付き暗号 (AEAD) の場合、認証タグや Nonce が結果に含まれることがあります。
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;

    /// 暗号文を共通鍵で復号します。
    /// 復号の過程でデータの改ざんチェック（認証）が行われます。
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;

    /// 現在設定されている共通鍵の生バイト列を取得します。
    /// 鍵交換プロトコルの過程で、相手に鍵を共有する際などに利用します。
    fn key_bytes(&self) -> Vec<u8>;
}
