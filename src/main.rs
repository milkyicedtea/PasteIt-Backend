use std::{env, fs};
use std::net::SocketAddr;
use std::path::Path;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::http::request::Parts;
use axum::{Router};
use deadpool_postgres::Pool;
use regex::Regex;
use tower_http::cors::{AllowOrigin, CorsLayer};
use crate::routes::pastes::pastes_router;
use crate::utils::database_config::get_db_pool;

mod utils;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool
}

async fn load_env() {
    let secret_path = "/run/secrets/";
    if Path::new(secret_path).exists() {
        println!("🔒 Loading environment variables from Docker secrets...");

        let secrets = ["DB_URL", "PASTE_ENCRYPTION_KEY", "RECAPTCHA_SECRET_KEY"];

        for secret in secrets.iter() {
            let secret_path = Path::new(secret_path).join(secret);
            if secret_path.exists() {
                if let Ok(value) = fs::read_to_string(&secret_path) {
                    env::set_var(secret.to_uppercase(), value.trim());
                }
            }
        }
    } else {
        println!("🛠️  Loading environment variables from .env...");
        dotenvy::dotenv().ok();
    }
}

fn create_cors_layer() -> CorsLayer {
    let origin_regex = if cfg!(debug_assertions) {
        Regex::new(r"(https?://)?(192)\.(168)\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9]){2}(?::\d+)?|localhost(?::\d+)?|127.0.0.1(?::\d+)?").unwrap()
    } else {
        Regex::new(r"^paste\.051205\.xyz$").unwrap()
    };

    CorsLayer::new()
        .allow_origin(AllowOrigin::async_predicate(move |origin: HeaderValue, _request_parts: &Parts| async move {
            let matches = origin_regex.is_match(origin.to_str().unwrap());
            println!("Origin: {:?}, Matches: {}", origin, matches);
            matches
        }))
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![
            "content-type",
            "origin"
        ].into_iter().map(|s| s.parse::<HeaderName>().unwrap()).collect::<Vec<_>>())
}

#[tokio::main]
async fn main() {
    load_env().await;

    let pool = get_db_pool().await;

    let state = AppState { pool };

    let app = Router::new()
        .nest("/api",
        Router::new()
            .nest("/pastes", pastes_router())
        )
        .layer(create_cors_layer())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
