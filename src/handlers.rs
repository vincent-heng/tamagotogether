use axum::{
    extract::{State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;
use crate::models::{AppState, StatusResponse, FeedResponse, Mood};

/// Helper to extract client IP, considering proxies.
fn get_client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string())
}

/// GET /api/state
pub async fn get_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ip = get_client_ip(&headers, addr);
    let level = state.db.get_level().unwrap_or(5);
    let mood = Mood::from_level(level);
    let has_fed_today = state.db.has_fed_today(&ip).unwrap_or(false);
    let feeds_today = state.db.get_feed_count_today().unwrap_or(0);
    
    Json(StatusResponse {
        level_id: level,
        mood_text: mood.as_text().to_string(),
        has_fed_today,
        feeds_today,
    })
}

/// POST /api/feed
pub async fn feed(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ip = get_client_ip(&headers, addr);
    let has_fed = state.db.has_fed_today(&ip).unwrap_or(false);
    
    if has_fed {
        let level = state.db.get_level().unwrap_or(5);
        let mood = Mood::from_level(level);
        let feeds_today = state.db.get_feed_count_today().unwrap_or(0);
        return (
            StatusCode::OK,
            Json(FeedResponse {
                message: "Tamagofox n'a plus faim mais mange quand même".to_string(),
                level_id: level,
                mood_text: mood.as_text().to_string(),
                feeds_today,
            }),
        ).into_response();
    }
    
    let old_level = state.db.get_level().unwrap_or(5);
    let new_level = match state.db.feed(&ip) {
        Ok(l) => l,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    let mood = Mood::from_level(new_level);
    let message = if new_level == 10 && old_level == 10 {
        "Tamagofox n'a plus faim mais mange quand même".to_string()
    } else {
        format!("Tamagofox mange et devient {}", mood.as_text())
    };

    let feeds_today = state.db.get_feed_count_today().unwrap_or(0);

    Json(FeedResponse {
        message,
        level_id: new_level,
        mood_text: mood.as_text().to_string(),
        feeds_today,
    }).into_response()
}
