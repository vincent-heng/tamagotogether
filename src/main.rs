use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod handlers;
mod models;

use db::Db;
use models::AppState;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

/// Handles serving static assets embedded in the binary.
async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            // Fallback to index.html for unknown paths to support frontend routing if needed
            match Asset::get("index.html") {
                Some(content) => {
                    let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                    ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                }
                None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tamagotogether=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "tamagotogether.db".to_string());
    let db = Db::new(&db_path).expect("Failed to initialize database");

    let state = AppState { db };

    let api_router = Router::new()
        .route("/state", get(handlers::get_status))
        .route("/feed", post(handlers::feed));

    let app = Router::new()
        .nest("/api", api_router)
        .fallback(static_handler)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let app_port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener_str = format!("0.0.0.0:{}", app_port);
    let listener = tokio::net::TcpListener::bind(&listener_str)
        .await
        .expect("Failed to bind port");
        
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Server failed");
}
