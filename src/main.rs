use std::net::SocketAddr;
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

fn create_cors_layer() -> CorsLayer {
    let origin_regex = Regex::new(r"(https?://)?(192)\.(168)\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9]){2}(?::\d+)?|localhost(?::\d+)?|127.0.0.1(?::\d+)?").unwrap();
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
    let pool = get_db_pool().await;

    let state = AppState { pool };

    let app = Router::new()
        .nest("/api",
        Router::new()
            .nest("/pastes", pastes_router())
        )
        .layer(create_cors_layer())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
