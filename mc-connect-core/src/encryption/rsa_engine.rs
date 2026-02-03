use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt, pkcs8::{EncodePublicKey, EncodePrivateKey}};
use rsa::signature::{Signer as RsaSignatureSigner, Verifier as RsaSignatureVerifier, SignatureEncoding};
use rsa::pkcs1v15::{SigningKey, VerifyingKey, Signature};
use rsa::sha2::Sha256;                 // ハッシュ関数アルゴリズム
use rand::rngs::OsRng;                // OSセキュアな乱数生成器
use std::error::Error;
use super::traits::{CryptoKeyPair, KeyGenerator, Encryptor, Signer};

/// [RsaKeyPair]
/// RSA (Rivest-Shamir-Adleman) アルゴリズムを使用したキーペアの実装です。
/// 公開鍵暗号方式により、データの暗号化・復号および署名の作成・検証の両方に対応します。
pub struct RsaKeyPair {
    /// 秘密鍵（自分だけが保持、復号と署名に使用）
    private_key: RsaPrivateKey,
    /// 公開鍵（相手に配布可能、暗号化と検証に使用）
    public_key: RsaPublicKey,
}

impl CryptoKeyPair for RsaKeyPair {
    /// アルゴリズムの識別名を返します。
    fn algorithm_name(&self) -> &str {
        "RSA"
    }

    /// 公開鍵を DER 形式 (SubjectPublicKeyInfo) のバイト列にエンコードして取得します。
    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.to_public_key_der().expect("RSA公開鍵のエンコードに失敗しました").to_vec()
    }

    /// 秘密鍵を DER 形式 (PKCS#8) のバイト列にエンコードして取得します。
    fn private_key_bytes(&self) -> Vec<u8> {
        self.private_key.to_pkcs8_der().expect("RSA秘密鍵のエンコードに失敗しました").to_bytes().to_vec()
    }
}

/// [RsaKeyGenerator]
/// RSA キーペアを特定のビット長で作成する生成器です。
pub struct RsaKeyGenerator {
    /// 生成する鍵のサイズ（ビット数）。デフォルトは 4096 です。
    pub bits: usize,
}

impl Default for RsaKeyGenerator {
    /// デフォルト設定: 現代的なセキュリティ基準を満たす 4096bit を採用します。
    fn default() -> Self {
        Self { bits: 4096 }
    }
}

impl KeyGenerator for RsaKeyGenerator {
    /// [generate]
    /// コンフィグに基づいた新しい RSA キーペアオブジェクトを作成します。
    fn generate(&self) -> Result<Box<dyn CryptoKeyPair>, Box<dyn Error>> {
        let mut rng = OsRng;
        // OSの乱数を使用して秘密鍵を新規作成
        let private_key = RsaPrivateKey::new(&mut rng, self.bits)?;
        // 秘密鍵から対応する公開鍵を抽出
        let public_key = RsaPublicKey::from(&private_key);
        Ok(Box::new(RsaKeyPair { private_key, public_key }))
    }
}

impl Encryptor for RsaKeyPair {
    /// [encrypt]
    /// PKCS#1 v1.5 に準拠した方式で公開鍵を用いてデータを暗号化します。
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut rng = OsRng;
        // Padding モードとして Pkcs1v15Encrypt を指定
        let enc_data = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
        Ok(enc_data)
    }

    /// [decrypt]
    /// 秘密鍵を用いて、暗号化されたデータを元の平文に戻します。
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // 暗号化時と同じ Padding 方式を使用して復号
        let dec_data = self.private_key.decrypt(Pkcs1v15Encrypt, data)?;
        Ok(dec_data)
    }
}

impl Signer for RsaKeyPair {
    /// [sign]
    /// RSASSA-PKCS1-v1_5 署名方式と SHA-256 ハッシュを使用して
    /// 指定されたデータに対するデジタル署名を作成します。
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // 署名用のキーをハッシュアルゴリズム SHA-256 と共に構築
        let signing_key = SigningKey::<Sha256>::new(self.private_key.clone());
        // データの署名を実行
        let signature = signing_key.sign(data);
        Ok(signature.to_vec())
    }

    /// [verify]
    /// 公開鍵を用いて、署名が正当であること（データが改ざんされておらず、
    /// 確かにペアの秘密鍵所有者によって署名されたこと）を確認します。
    fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> Result<bool, Box<dyn Error>> {
        // 検証用のキーを公開鍵から構築
        let verifying_key = VerifyingKey::<Sha256>::new(self.public_key.clone());
        // バイト列を署名型に変換
        let signature = Signature::try_from(signature_bytes)
            .map_err(|_| "署名のフォーマットが不正です。")?;
        
        // 検証を実行。成功すれば Ok(()) が返るため is_ok() で判定
        Ok(verifying_key.verify(data, &signature).is_ok())
    }
}
