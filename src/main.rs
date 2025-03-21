use std::net::SocketAddr;
use axum::{Router};
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};
use crate::routes::pastes::pastes_router;
use crate::utils::appstate::AppState;
use crate::utils::cors::create_cors_layer;
use crate::utils::database_config::get_db_pool;
use crate::utils::env::load_env;
use crate::utils::swagger::ApiDoc;

mod utils;
mod routes;



#[tokio::main]
async fn main() {
    load_env().await;

    let pool = get_db_pool("DB_URL").await;

    let state = AppState { pool };

    let app = Router::new()
        .nest("/api",
        Router::new()
            .nest("/pastes", pastes_router())
            .route("/test", get(|| async {
                "Hello World!"
            }))
        )
        .merge(SwaggerUi::new("/api/docs")
            .url("/api/docs/openapi.json", ApiDoc::openapi())
            .config(Config::default()
                .use_base_layout()
                .try_it_out_enabled(false)
                .doc_expansion("list")
                .with_syntax_highlight(true)
            )
        )
        .layer(create_cors_layer())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
