use axum::{
    extract::{State, ConnectInfo, Query},
    http::{StatusCode, HeaderMap, header},
    response::{IntoResponse, Redirect},
    Json,
};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    AuthorizationCode, TokenResponse, Scope, CsrfToken,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::models::{AppState, StatusResponse, FeedResponse, PlayResponse, Mood, Playfulness, User};

/// GET /api/auth/discord/login
pub async fn discord_login(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = match state.discord_config {
        Some(c) => c,
        None => return StatusCode::NOT_IMPLEMENTED.into_response(),
    };

    let client = BasicClient::new(
        ClientId::new(config.client_id),
        Some(ClientSecret::new(config.client_secret)),
        AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_url).unwrap());

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .url();

    Redirect::to(auth_url.as_str()).into_response()
}

#[derive(serde::Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

/// GET /api/auth/discord/callback
pub async fn discord_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let config = match state.discord_config {
        Some(c) => c,
        None => return StatusCode::NOT_IMPLEMENTED.into_response(),
    };

    let client = BasicClient::new(
        ClientId::new(config.client_id),
        Some(ClientSecret::new(config.client_secret)),
        AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_url).unwrap());

    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    let token = match token_result {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to exchange token: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Get user info from Discord
    let client = reqwest::Client::new();
    let user_info_res = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .send()
        .await;

    let user: User = match user_info_res {
        Ok(res) => res.json().await.unwrap(),
        Err(e) => {
            tracing::error!("Failed to get user info: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Store user in DB and create session
    let session_id = match state.db.create_session(&user) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to create session: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let cookie = format!("session_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000", session_id);
    
    (
        StatusCode::FOUND,
        [(header::SET_COOKIE, cookie), (header::LOCATION, "/".to_string())],
    ).into_response()
}

/// GET /api/auth/me
pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .find(|s| s.trim().starts_with("session_id="))
                .map(|s| s.trim()["session_id=".len()..].to_string())
        });

    let session_id = match cookie {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    match state.db.get_user_by_session(&session_id) {
        Ok(Some(user)) => Json(user).into_response(),
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Helper to extract client IP, considering proxies.
fn get_client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string())
}

/// Helper to extract session ID from cookies.
fn get_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .find(|s| s.trim().starts_with("session_id="))
                .map(|s| s.trim()["session_id=".len()..].to_string())
        })
}

/// GET /api/state
pub async fn get_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let lang = params.get("lang").map(|s| s.as_str()).unwrap_or("fr");
    let ip = get_client_ip(&headers, addr);
    
    let session_id = get_session_id(&headers);
    let user = session_id.and_then(|id| state.db.get_user_by_session(&id).ok().flatten());
    let user_id = user.as_ref().map(|u| u.id.as_str());

    let level = state.db.get_level().unwrap_or(5);
    let mood = Mood::from_level(level);
    let has_fed_today = state.db.has_fed_today(&ip, user_id).unwrap_or(false);
    let feeds_today = state.db.get_feed_count_today().unwrap_or(0);

    let player_plays_today = state.db.get_player_play_count_today(&ip, user_id).unwrap_or(0);
    let plays_today = state.db.get_play_count_today().unwrap_or(0);
    let playfulness_level = state.db.get_playfulness_level().unwrap_or(1);
    let playfulness = Playfulness::from_level(playfulness_level);
    let can_play = level == 10 && player_plays_today < 3;

    Json(StatusResponse {
        level_id: level,
        mood_text: mood.as_text(lang).to_string(),
        has_fed_today,
        feeds_today,
        can_play,
        player_plays_today,
        plays_today,
        playfulness_text: playfulness.as_text(lang).to_string(),
        playfulness_level,
        user_coins: user.map(|u| u.coins),
    })
}

/// POST /api/feed
pub async fn feed(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let lang = params.get("lang").map(|s| s.as_str()).unwrap_or("fr");
    let ip = get_client_ip(&headers, addr);
    
    let session_id = get_session_id(&headers);
    let user = session_id.and_then(|id| state.db.get_user_by_session(&id).ok().flatten());
    let user_id = user.as_ref().map(|u| u.id.as_str());

    let has_fed = state.db.has_fed_today(&ip, user_id).unwrap_or(false);
    
    if has_fed {
        let level = state.db.get_level().unwrap_or(5);
        let mood = Mood::from_level(level);
        let feeds_today = state.db.get_feed_count_today().unwrap_or(0);
        let msg = match lang {
            "en" => "Tamagofox is not hungry anymore but eats anyway",
            "de" => "Tamagofox hat keinen Hunger mehr, isst aber trotzdem",
            _ => "Tamagofox n'a plus faim mais mange quand même",
        };
        return (
            StatusCode::OK,
            Json(FeedResponse {
                message: msg.to_string(),
                level_id: level,
                mood_text: mood.as_text(lang).to_string(),
                feeds_today,
                user_coins: user.map(|u| u.coins),
            }),
        ).into_response();
    }
    
    let old_level = state.db.get_level().unwrap_or(5);
    let new_level = match state.db.feed(&ip, user_id) {
        Ok(l) => l,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    // Refresh user to get new coins
    let user = session_id.and_then(|id| state.db.get_user_by_session(&id).ok().flatten());
    
    let mood = Mood::from_level(new_level);
    let message = if new_level == 10 && old_level == 10 {
        match lang {
            "en" => "Tamagofox is not hungry anymore but eats anyway".to_string(),
            "de" => "Tamagofox hat keinen Hunger mehr, isst aber trotzdem".to_string(),
            _ => "Tamagofox n'a plus faim mais mange quand même".to_string(),
        }
    } else {
        match lang {
            "en" => format!("Tamagofox eats and becomes {}", mood.as_text(lang)),
            "de" => format!("Tamagofox isst und wird {}", mood.as_text(lang)),
            _ => format!("Tamagofox mange et devient {}", mood.as_text(lang)),
        }
    };

    let feeds_today = state.db.get_feed_count_today().unwrap_or(0);

    Json(FeedResponse {
        message,
        level_id: new_level,
        mood_text: mood.as_text(lang).to_string(),
        feeds_today,
        user_coins: user.map(|u| u.coins),
    }).into_response()
}

/// POST /api/play
pub async fn play(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let lang = params.get("lang").map(|s| s.as_str()).unwrap_or("fr");
    let ip = get_client_ip(&headers, addr);
    
    let session_id = get_session_id(&headers);
    let user = session_id.and_then(|id| state.db.get_user_by_session(&id).ok().flatten());
    let user_id = user.as_ref().map(|u| u.id.as_str());

    // Only allow playing when happiness is at max
    let level = state.db.get_level().unwrap_or(5);
    if level < 10 {
        return StatusCode::FORBIDDEN.into_response();
    }

    let player_plays = state.db.get_player_play_count_today(&ip, user_id).unwrap_or(0);
    let old_playfulness = state.db.get_playfulness_level().unwrap_or(1);

    // Player already used all 3 plays
    if player_plays >= 3 {
        let playfulness = Playfulness::from_level(old_playfulness);
        let plays_today = state.db.get_play_count_today().unwrap_or(0);
        let msg = match lang {
            "en" => "Tamagofox doesn't want to play anymore but plays anyway",
            "de" => "Tamagofox möchte nicht mehr spielen, spielt aber trotzdem",
            _ => "Tamagofox n'a plus envie de jouer mais joue quand même",
        };
        return Json(PlayResponse {
            message: msg.to_string(),
            playfulness_text: playfulness.as_text(lang).to_string(),
            playfulness_level: old_playfulness,
            plays_today,
            player_plays_today: player_plays,
            user_coins: user.map(|u| u.coins),
        }).into_response();
    }

    let new_playfulness = match state.db.play(&ip, user_id) {
        Ok(l) => l,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // Refresh user to get new coins
    let user = session_id.and_then(|id| state.db.get_user_by_session(&id).ok().flatten());

    let playfulness = Playfulness::from_level(new_playfulness);
    let plays_today = state.db.get_play_count_today().unwrap_or(0);
    let player_plays_after = state.db.get_player_play_count_today(&ip, user_id).unwrap_or(0);

    let message = if new_playfulness == 10 && old_playfulness == 10 {
        match lang {
            "en" => "Tamagofox doesn't want to play anymore but plays anyway".to_string(),
            "de" => "Tamagofox möchte nicht mehr spielen, spielt aber trotzdem".to_string(),
            _ => "Tamagofox n'a plus envie de jouer mais joue quand même".to_string(),
        }
    } else if new_playfulness != old_playfulness {
        match lang {
            "en" => format!("Tamagofox plays and becomes {}", playfulness.as_text(lang)),
            "de" => format!("Tamagofox spielt und wird {}", playfulness.as_text(lang)),
            _ => format!("Tamagofox joue et devient {}", playfulness.as_text(lang)),
        }
    } else {
        match lang {
            "en" => "Tamagofox plays".to_string(),
            "de" => "Tamagofox spielt".to_string(),
            _ => "Tamagofox joue".to_string(),
        }
    };

    Json(PlayResponse {
        message,
        playfulness_text: playfulness.as_text(lang).to_string(),
        playfulness_level: new_playfulness,
        plays_today,
        player_plays_today: player_plays_after,
        user_coins: user.map(|u| u.coins),
    }).into_response()
}
