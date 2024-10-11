// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use axum::{
    extract::{Form, Path, Request, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Json, Response},
    routing::{get, post, Router},
};

use serde_json::json;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

use tower_http::trace::TraceLayer;
use tracing::info;

use crate::utils::axum::AxumResult;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ValfiskPresenceData {
    pub status: serenity::OnlineStatus,
    pub client_status: Option<serenity::ClientStatus>,
    #[serde(default)]
    pub activities: Vec<serenity::Activity>,
}

impl ValfiskPresenceData {
    pub fn from_presence(presence: &serenity::Presence) -> Self {
        Self {
            status: presence.status,
            client_status: presence.client_status.clone(),
            activities: presence.activities.to_vec(),
        }
    }
}

pub static PRESENCE_STORE: Lazy<RwLock<HashMap<serenity::UserId, ValfiskPresenceData>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

async fn route_ping() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "ok": true })))
}

async fn route_ping_head() -> impl IntoResponse {
    StatusCode::OK
}

async fn route_not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({ "error": "Not found" })))
}

async fn route_presence(Path(user_id): Path<u64>) -> AxumResult<Response> {
    if user_id == 0 {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "User ID cannot be 0" })),
        )
            .into_response());
    }

    let user_id = serenity::UserId::from(user_id);

    let store = PRESENCE_STORE.read().unwrap();
    let presence_data = store.get(&user_id).cloned();
    drop(store);

    presence_data.map_or_else(
        || {
            Ok((
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "User not found" })),
            )
                .into_response())
        },
        |presence_data| Ok((StatusCode::OK, Json(presence_data)).into_response()),
    )
}

async fn route_presence_head(Path(user_id): Path<u64>) -> AxumResult<StatusCode> {
    if user_id == 0 {
        return Ok(StatusCode::BAD_REQUEST);
    }

    let user_id = serenity::UserId::from(user_id);

    let store = PRESENCE_STORE.read().unwrap();
    let presence_exists = store.contains_key(&user_id);
    drop(store);

    if presence_exists {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

#[derive(serde::Deserialize, Debug)]
struct KofiFormData {
    data: String,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct KofiData {
    verification_token: String,
    r#type: String,
    is_public: bool,
    from_name: String,
    message: Option<String>,
    amount: String,
    currency: String,
    timestamp: serenity::Timestamp,
}

async fn route_kofi_webhook(
    State(state): State<Arc<AppState>>,
    form: Form<KofiFormData>,
) -> AxumResult<(StatusCode, impl IntoResponse)> {
    let data: KofiData = serde_json::from_str(&form.0.data)?;
    let verification_token = std::env::var("KOFI_VERIFICATION_TOKEN")?;

    if data.verification_token != verification_token {
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized" })),
        ));
    }

    if data.is_public {
        if let Some(channel) = std::env::var("KOFI_NOTIFY_CHANNEL")
            .ok()
            .and_then(|c| c.parse::<u64>().ok())
            .map(serenity::ChannelId::new)
        {
            let mut embed = serenity::CreateEmbed::default()
                .title(format!("Thank you to {}!", data.from_name))
                .description(format!(
                    "For donating **{} {}** 🥳",
                    data.amount, data.currency
                ))
                .timestamp(data.timestamp)
                .color(0xffd43b);

            if let Some(message) = data.message {
                embed = embed.field("Message", message, false);
            }

            channel
                .send_message(
                    &state.serenity_http,
                    serenity::CreateMessage::default().embed(embed),
                )
                .await?;
        }
    }

    Ok((StatusCode::OK, Json(json!({ "ok": true }))))
}

#[derive(Debug)]
struct AppState {
    serenity_http: Arc<serenity::Http>,
}

async fn security_middleware(request: Request, next: middleware::Next) -> Response {
    let mut response = next.run(request).await;

    let h = response.headers_mut();
    h.insert(
        "content-security-policy",
        "default-src 'none'".parse().unwrap(),
    );
    h.insert("access-control-allow-origin", "*".parse().unwrap());
    h.insert("cross-origin-opener-policy", "same-origin".parse().unwrap());
    h.insert(
        "cross-origin-resource-policy",
        "same-origin".parse().unwrap(),
    );
    h.insert("origin-agent-cluster", "?1".parse().unwrap());
    h.insert("referrer-policy", "no-referrer".parse().unwrap());
    h.insert("x-content-type-options", "nosniff".parse().unwrap());
    h.insert("x-dns-prefetch-control", "off".parse().unwrap());
    h.insert("x-download-options", "noopen".parse().unwrap());
    h.insert("x-frame-options", "DENY".parse().unwrap());
    h.insert("x-permitted-cross-domain-policies", "none".parse().unwrap());
    h.insert("x-xss-protection", "1; mode=block".parse().unwrap());

    response
}

#[tracing::instrument(skip(serenity_http))]
pub async fn serve(serenity_http: Arc<serenity::Http>) -> eyre::Result<()> {
    #[cfg(debug_assertions)]
    let default_host = "127.0.0.1";
    #[cfg(not(debug_assertions))]
    let default_host = "0.0.0.0";

    let host = std::env::var("HOST").unwrap_or_else(|_| default_host.to_owned());
    let port = std::env::var("PORT").map_or(Ok(8080), |v| v.parse::<u16>())?;

    let state = Arc::new(AppState { serenity_http });

    info!("Started API server {}", format!("http://{host}:{port}"));

    let listener = TcpListener::bind((host, port)).await?;

    let app = Router::new()
        .route("/", get(route_ping).head(route_ping_head))
        .route(
            "/presence/:user",
            get(route_presence).head(route_presence_head),
        )
        .route("/ko-fi", post(route_kofi_webhook))
        .fallback(route_not_found)
        .layer(middleware::from_fn(security_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::serve(listener, app).await?;

    Ok(())
}
