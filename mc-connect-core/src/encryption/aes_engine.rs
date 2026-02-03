use aes_gcm::{
    aead::{Aead, KeyInit, AeadCore},
    Aes256Gcm, Key, Nonce
};
use rand::RngCore;                   // 乱数生成のためのインターフェース
use rand::rngs::OsRng;                // OS標準のセキュアな乱数生成器
use std::error::Error;
use super::traits::SymmetricCrypto;   // 共通鍵暗号の抽象トレイト

/// [AesGcmEngine]
/// AES (Advanced Encryption Standard) の GCM (Galois/Counter Mode) モードを使用した実装です。
/// 
/// AES-256-GCM は、データの暗号化（機密性）と同時に、改ざん検知（認証性）も行うことができる
/// 「認証付き暗号 (AEAD)」のデファクトスタンダードです。
pub struct AesGcmEngine {
    /// 内部で使用する暗号化オブジェクト（aes-gcm クレートのもの）
    cipher: Aes256Gcm,
    /// 保持している共通鍵（32バイト / 256ビット）
    key: Vec<u8>,
}

impl AesGcmEngine {
    /// [new_random]
    /// 完全にランダムな 256ビット (32バイト) の鍵を新規に生成して、
    /// 新しい暗号化エンジン・インスタンスを作成します。
    pub fn new_random() -> Self {
        let mut key_bytes = [0u8; 32]; // 32バイトのバッファを用意
        OsRng.fill_bytes(&mut key_bytes); // セキュアな乱数で埋める
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes); // 型安全な Key オブジェクトに変換
        let cipher = Aes256Gcm::new(key); // AES-GCM インスタンスを初期化
        
        Self {
            cipher,
            key: key_bytes.to_vec(), // 内部保存用に Vec に変換
        }
    }

    /// [from_key]
    /// 外部から提供されたバイト列（鍵交換の結果得られたもの等）を使用して、
    /// 暗号化エンジンを初期化します。
    pub fn from_key(key_bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        // AES-256 のため、鍵は必ず 32バイト でなければなりません
        if key_bytes.len() != 32 {
            return Err("AES-256 の鍵は正確に 32バイトである必要があります。".into());
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes); // スライスから鍵オブジェクトを作成
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self {
            cipher,
            key: key_bytes.to_vec(),
        })
    }
}

/// 抽象インターフェース `SymmetricCrypto` の実装
impl SymmetricCrypto for AesGcmEngine {
    /// [encrypt]
    /// 指定されたデータを暗号化し、Nonce と暗号文が結合されたデータを返します。
    /// 
    /// セキュリティ上の注意：
    /// 同じ鍵で同じ Nonce (Number used once) を二度使ってはいけません。
    /// そのため、この実装では暗号化のたびに新しい 12バイトの Nonce をランダムに生成します。
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // 1. 毎回異なる 12バイトの Nonce を生成
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // 2. 暗号化処理の実行。成功すると暗号文が返ります。
        // ※ aes-gcm クレートの encrypt メソッドを使用
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)
            .map_err(|e| format!("AES-GCM 暗号化に失敗しました: {}", e))?;

        // 3. 復号側が同じ Nonce を使えるように、[Nonce (12B)] + [Ciphertext] の形で結合
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);      // Nonce を先頭に追加
        result.extend_from_slice(&ciphertext); // 続けて暗号文を追加
        
        Ok(result)
    }

    /// [decrypt]
    /// 暗号化されたバイト列を受け取り、Nonce を取り出して復号を行います。
    /// 
    /// 認証 (AEAD) について：
    /// もしデータの一部が 1ビットでも書き換えられていた場合、
    /// `decrypt` メソッドはエラーとなり、不正なデータが返されることはありません。
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // Nonce 分の 12バイトすら無い場合はエラー
        if data.len() < 12 {
            return Err("暗号文が短すぎます。Nonce (12バイト) が含まれていません。".into());
        }

        // 1. 先頭 12バイトを Nonce として取り出し、残りを暗号文として扱う
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes); // 正しい型に変換

        // 2. 復号処理の実行
        // 鍵・Nonce・暗号文が全て揃っており、かつ改ざんがない場合のみ成功します。
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("AES-GCM 復号に失敗しました: {} (データが改ざんされている可能性があります)", e))?;

        Ok(plaintext)
    }

    /// [key_bytes]
    /// このエンジンが現在暗号化に使用している共通鍵のバイト列を返します。
    fn key_bytes(&self) -> Vec<u8> {
        self.key.clone()
    }
}
