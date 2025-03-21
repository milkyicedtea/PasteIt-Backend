use axum::http::{HeaderName, HeaderValue, Method};
use axum::http::request::Parts;
use regex::Regex;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub(crate) fn create_cors_layer() -> CorsLayer {
    let origin_regex = if cfg!(debug_assertions) {
        println!("Using debug regex");
        Regex::new(r"(https?://)?(192)\.(168)\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9]){2}(?::\d+)?|localhost(?::\d+)?|127.0.0.1(?::\d+)?|.*\.localhost(?::\d+)?").unwrap()
    } else {
        println!("Using production regex");
        Regex::new(r"^(?:https?://(?:.*\.)?051205\.xyz(?::\d+)?|https?://[\w.-]+:\d+)$").unwrap()
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