use crate::models::packet::{Message, Command, SecureConnectPayload, Protocol};
use crate::encryption::{Encryptor, SymmetricCrypto, AesGcmEngine, traits::CryptoError};

/// [SecureContext]
/// 暗号化セッションの状態を管理し、メッセージの暗号化・復号を行います。
pub struct SecureContext {
    /// 確立された共通鍵暗号エンジン。ハンドシェイク前は None です。
    pub crypto: Option<Box<dyn SymmetricCrypto>>,
}

impl SecureContext {
    /// 空のコンテキスト（未初期化状態）を作成します。
    pub fn new() -> Self {
        Self { crypto: None }
    }

    /// [seal_message]
    /// メッセージのペイロード部分を暗号化します。
    pub fn seal_message(&self, mut msg: Message) -> Result<Message, CryptoError> {
        if let Some(crypto) = &self.crypto {
            msg.payload = crypto.encrypt(&msg.payload)?;
        }
        Ok(msg)
    }

    /// [unseal_message]
    /// 暗号化されたメッセージのペイロードを復号します。
    pub fn unseal_message(&self, mut msg: Message) -> Result<Message, CryptoError> {
        if let Some(crypto) = &self.crypto {
            msg.payload = crypto.decrypt(&msg.payload)?;
        }
        Ok(msg)
    }
}

/// [handle_server_handshake]
/// サーバー側でのセキュアハンドシェイク（同期処理）。
pub fn handle_server_handshake(
    raw_packet: Message,
    server_key_pair: &dyn Encryptor,
) -> Result<(SecureContext, Protocol, u16), CryptoError> {
    
    if raw_packet.command != Command::SecureConnect {
        return Err("初期パケットは SecureConnect である必要があります。".into());
    }

    let payload: SecureConnectPayload = raw_packet.deserialize_payload()
        .map_err(|e| format!("SecureConnect ペイロードの解析に失敗しました: {}", e))?;

    let symmetric_key = server_key_pair.decrypt(&payload.encrypted_key)
        .map_err(|e| format!("対称鍵の復号に失敗しました: {}", e))?;

    let crypto = AesGcmEngine::from_key(&symmetric_key)?;

    let mut context = SecureContext::new();
    context.crypto = Some(Box::new(crypto));

    Ok((context, payload.protocol, payload.port))
}

/// [create_secure_connect_packet]
/// クライアント側でのセキュア接続要求の構築。
pub fn create_secure_connect_packet(
    protocol: Protocol,
    port: u16,
    server_public_key: &dyn Encryptor,
) -> Result<(SecureContext, Message), CryptoError> {
    
    let aes_engine = AesGcmEngine::new_random();
    let key_bytes = aes_engine.key_bytes();

    let encrypted_key = server_public_key.encrypt(&key_bytes)?;

    let payload = SecureConnectPayload {
        protocol,
        port,
        encrypted_key,
        algorithm: "AES-256-GCM".to_string(),
    };

    let msg = Message::from_payload(Command::SecureConnect, &payload)
        .map_err(|e| format!("ハンドシェイクパケットの生成に失敗しました: {}", e))?;
    
    let mut context = SecureContext::new();
    context.crypto = Some(Box::new(aes_engine));

    Ok((context, msg))
}
