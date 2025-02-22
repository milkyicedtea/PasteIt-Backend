use std::net::{SocketAddr};
use aes_gcm::{Aes256Gcm, Key};
use axum::extract::{ConnectInfo, Path, State};
use axum::{Json, Router};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::AppState;
use crate::utils::cryptography::{decrypt_paste, encrypt_paste, get_key_bytes};
use crate::utils::net_utils::{check_rate_limit, get_real_ip};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Paste {
    name: Option<String>,
    paste: String,
    language: String,
    recaptcha_token: String
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PasteResponse {
    short_id: String    // Base62-encoded ID
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PasteContentResponse {
    name: Option<String>,
    paste: String,
    language: Option<String>,
    created_at: DateTime<Utc>
}

pub fn pastes_router() -> Router<AppState> {
    Router::new()
        .route("/paste", post(create_paste))
        .route("/paste/{short_id}", get(get_paste))
}

async fn create_paste(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(paste_data): Json<Paste>
) -> Result<Json<PasteResponse>, (StatusCode, String)> {

    let is_human = verify_recaptcha(&paste_data.recaptcha_token).await?;
    if !is_human {
        return Err((StatusCode::FORBIDDEN, "reCAPTCHA verification failed".to_string()));
    }

    let db = state.pool.get().await.unwrap();

    let client_ip = get_real_ip(&headers, &addr).await;

    // check rate limit before creating paste
    if let Err(err) = check_rate_limit(&state.pool, client_ip).await {
        return Err(err)
    }

    // load encryption key
    let key_bytes = get_key_bytes().await?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let encrypted_paste = encrypt_paste(&paste_data.paste, key).await?;

    let query = db.prepare_cached(
        r#"
        insert into pastes (name, paste, language, created_at)
        values ($1, $2, $3, now())
        returning id
        "#).await.unwrap();

    let id: i64 = db.query_one(
        &query, &[&paste_data.name.as_deref().unwrap_or("Untitled PasteIt").chars().take(128).collect::<String>(),
            &encrypted_paste, &paste_data.language]).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Exec Error: {}", e))
        })?.get("id");

    let short_id = bs62::encode_num(&id);
    println!("create_paste: {:?} with short id {:?}", &id, &short_id);


    Ok(Json(PasteResponse { short_id }))
}

async fn get_paste(
    State(state): State<AppState>,
    Path(short_id): Path<String>,
) -> Result<Json<PasteContentResponse>, (StatusCode, String)> {
    println!("Get paste called!");
    println!("Get paste: {:?}", short_id);

    let big_uint = bs62::decode_num(&short_id).map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Invalid short ID: {}", e))
    })?;

    let id: i64 = big_uint.try_into().map_err(|_| {
        (StatusCode::BAD_REQUEST, "ID out of range".to_string())
    })?;

    let db = state.pool.get().await.unwrap();
    let query = db.prepare_cached(
        r#"
        select name, paste, language, created_at from pastes
        where id = $1
        "#).await.unwrap();

    let paste = db.query_one(&query, &[&id]).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Exec Error: {}", e))
    })?;

    let encrypted_paste: String = paste.get("paste");

    // Load AES encryption key from environment variable
    let key_bytes = get_key_bytes().await?;

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

    let decrypted_paste = decrypt_paste(&encrypted_paste, key).await?;

    Ok(Json(PasteContentResponse{
        name: paste.get("name"),
        paste: decrypted_paste,
        language: paste.get("language"),
        created_at: paste.get("created_at")
    }))
}

async fn verify_recaptcha(token: &str) -> Result<bool, (StatusCode, String)> {
    let client = Client::new();
    let recaptcha_secret = std::env::var("RECAPTCHA_SECRET_KEY").map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "reCAPTCHA secret key not set".to_string())
    })?;

    let params = [
        ("secret", recaptcha_secret),
        ("response", token.to_string())
    ];

    let response = client.post("https://www.google.com/recaptcha/api/siteverify")
        .form(&params)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("reCAPTCHA verification failed: {}", e)))?;

    let result: Value = response.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Invalid reCAPTCHA response: {}", e)))?;

    if result["success"].as_bool().unwrap_or(false) {
        let score = result["score"].as_f64().unwrap_or(0.0);

        println!("human score: {}", score);
        if score >= 0.3 {
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}