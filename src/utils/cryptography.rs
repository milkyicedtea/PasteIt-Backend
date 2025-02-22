use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng, rand_core::RngCore};
use axum::http::StatusCode;

pub async fn encrypt_paste(plain_text: &str, key: &Key<Aes256Gcm>) -> Result<String, (StatusCode, String)> {
    let cipher = Aes256Gcm::new(key);

    // generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // encrypt the paste
    let ciphertext = cipher.encrypt(nonce, plain_text.as_bytes()).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Encryption Error: {}", e))
    })?;

    // store nonce + encrypted text together
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))   // hex encoded for storage
}

pub async fn decrypt_paste(encrypted_text: &str, key: &Key<Aes256Gcm>) -> Result<String, (StatusCode, String)> {
    let cipher = Aes256Gcm::new(key);

    let data = hex::decode(encrypted_text).map_err(|e| (StatusCode::BAD_REQUEST, "Invalid hex encoding".to_owned() + &*e.to_string()))?;

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Decryption Error: {}", e))
    })?;

    Ok(String::from_utf8(plaintext).unwrap())
}

pub async fn get_key_bytes() -> Result<Vec<u8>, (StatusCode, String)> {
    hex::decode(std::env::var("PASTE_ENCRYPTION_KEY").map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Encryption key not set".to_string())
    })?).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Invalid key format".to_string())
    })
}