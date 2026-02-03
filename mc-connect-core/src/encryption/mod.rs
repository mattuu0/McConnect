pub mod traits;
pub mod rsa_engine;
pub mod aes_engine;
pub mod secure_connect;

pub use traits::{CryptoKeyPair, KeyGenerator, Encryptor, Signer, SymmetricCrypto, CryptoError};
pub use rsa_engine::{RsaKeyPair, RsaKeyGenerator};
pub use aes_engine::AesGcmEngine;
pub use secure_connect::{SecureContext, handle_server_handshake, create_secure_connect_packet};

/// アルゴリズムの種類を指定する列挙型。
/// 将来的に ED25519 等を追加できるように設計されています。
pub enum Algorithm {
    Rsa,
}

/// [create_generator]
/// 指定されたアルゴリズムに対応するキー生成器を作成します。
pub fn create_generator(algo: Algorithm) -> Box<dyn KeyGenerator> {
    match algo {
        Algorithm::Rsa => Box::new(RsaKeyGenerator::default()),
    }
}
