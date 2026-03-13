use axum::{
    extract::{State, ConnectInfo},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;
use crate::models::{AppState, StatusResponse, FeedResponse, PlayResponse, Mood, Playfulness};

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

    let player_plays_today = state.db.get_player_play_count_today(&ip).unwrap_or(0);
    let plays_today = state.db.get_play_count_today().unwrap_or(0);
    let playfulness_level = state.db.get_playfulness_level().unwrap_or(1);
    let playfulness = Playfulness::from_level(playfulness_level);
    let can_play = level == 10 && player_plays_today < 3;

    Json(StatusResponse {
        level_id: level,
        mood_text: mood.as_text().to_string(),
        has_fed_today,
        feeds_today,
        can_play,
        player_plays_today,
        plays_today,
        playfulness_text: playfulness.as_text().to_string(),
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

/// POST /api/play
pub async fn play(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ip = get_client_ip(&headers, addr);

    // Only allow playing when happiness is at max
    let level = state.db.get_level().unwrap_or(5);
    if level < 10 {
        return StatusCode::FORBIDDEN.into_response();
    }

    let player_plays = state.db.get_player_play_count_today(&ip).unwrap_or(0);
    let old_playfulness = state.db.get_playfulness_level().unwrap_or(1);

    // Player already used all 3 plays
    if player_plays >= 3 {
        let playfulness = Playfulness::from_level(old_playfulness);
        let plays_today = state.db.get_play_count_today().unwrap_or(0);
        return Json(PlayResponse {
            message: "Tamagofox n'a plus envie de jouer mais joue quand même".to_string(),
            playfulness_text: playfulness.as_text().to_string(),
            plays_today,
            player_plays_today: player_plays,
        }).into_response();
    }

    let new_playfulness = match state.db.play(&ip) {
        Ok(l) => l,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let playfulness = Playfulness::from_level(new_playfulness);
    let plays_today = state.db.get_play_count_today().unwrap_or(0);
    let player_plays_after = state.db.get_player_play_count_today(&ip).unwrap_or(0);

    let message = if new_playfulness == 10 && old_playfulness == 10 {
        "Tamagofox n'a plus envie de jouer mais joue quand même".to_string()
    } else if new_playfulness != old_playfulness {
        format!("Tamagofox joue et devient {}", playfulness.as_text())
    } else {
        "Tamagofox joue".to_string()
    };

    Json(PlayResponse {
        message,
        playfulness_text: playfulness.as_text().to_string(),
        plays_today,
        player_plays_today: player_plays_after,
    }).into_response()
}
