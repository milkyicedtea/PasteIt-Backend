use std::net::{IpAddr, SocketAddr};
use axum::http::{HeaderMap, StatusCode};
use deadpool_postgres::Pool;
use sha2::{Sha256, Digest};

pub(crate) async fn get_real_ip(headers: &HeaderMap, connect_info: &SocketAddr) -> IpAddr {
    if let Some(fowarded_for) = headers.get("x-forwarded-for") {
        if let Ok(ip_str) = fowarded_for.to_str() {
            if let Ok(ip) = ip_str.split(',').next().unwrap_or("").trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    connect_info.ip()
}

pub(crate) async fn check_rate_limit(pool: &Pool, ip: IpAddr, ) -> Result<(), (StatusCode, String)> {
    let db = pool.get().await.unwrap();

    let hashed_ip = hash_ip(&ip).await;

    let query = db.prepare_cached(
        r#"
        with upsert as (
            insert into paste_rate_limits (encrypted_ip, paste_count, last_reset)
            values ($1, 1, current_timestamp)
            on conflict (encrypted_ip) do update set
                paste_count = case
                    when paste_rate_limits.last_reset < current_timestamp - interval '24 hours'
                    then 1
                    else paste_rate_limits.paste_count + 1
                end,
                last_reset = case
                    when paste_rate_limits.last_reset < current_timestamp - interval '24 hours'
                    then current_timestamp
                    else paste_rate_limits.last_reset
                end
            returning paste_count
        )
        select paste_count from upsert
        "#
    ).await.unwrap();

    let result: i32 = db.query_one(&query, &[&hashed_ip]).await.map_err(
        |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Rate limit check failed: {}", e)))?
        .get("paste_count");

    println!("paste_count: {}", result);

    if result > 5 {
        println!("returning error");
        return Err((StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded. Maximum 5 pastes per day.".to_string()));
    }

    Ok(())
}

async fn hash_ip(ip: &IpAddr) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ip.to_string().as_bytes());
    hex::encode(hasher.finalize())
}