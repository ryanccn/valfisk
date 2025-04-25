// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use axum::{
    extract::Request,
    middleware,
    response::Response,
    routing::{Router, get, post},
};

use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::CONFIG;

mod routes {
    use serde_json::json;
    use std::sync::Arc;

    use axum::{
        extract::{Form, State},
        http::StatusCode,
        response::{IntoResponse, Json},
    };
    use poise::serenity_prelude as serenity;

    use crate::{
        config::CONFIG,
        utils::{AxumResult, truncate},
    };

    #[tracing::instrument(skip_all)]
    pub async fn ping() -> impl IntoResponse {
        (StatusCode::OK, Json(json!({ "ok": true })))
    }

    #[tracing::instrument(skip_all)]
    pub async fn ping_head() -> impl IntoResponse {
        StatusCode::OK
    }

    #[tracing::instrument(skip_all)]
    pub async fn not_found() -> impl IntoResponse {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Not found" })))
    }

    #[derive(serde::Deserialize, Debug)]
    pub struct KofiFormData {
        data: String,
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    pub struct KofiData {
        verification_token: String,
        r#type: String,
        is_public: bool,
        from_name: String,
        message: Option<String>,
        amount: String,
        currency: String,
        timestamp: serenity::Timestamp,
    }

    #[tracing::instrument(skip_all)]
    pub async fn kofi_webhook(
        State(state): State<Arc<super::AppState>>,
        form: Form<KofiFormData>,
    ) -> AxumResult<(StatusCode, impl IntoResponse)> {
        let data: KofiData = serde_json::from_str(&form.0.data)?;

        if Some(data.verification_token) != CONFIG.kofi_verification_token {
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Unauthorized" })),
            ));
        }

        if data.is_public {
            if let Some(channel) = CONFIG.kofi_notify_channel {
                let mut embed = serenity::CreateEmbed::default()
                    .title(format!("Thank you to {}!", data.from_name))
                    .description(format!(
                        "For donating **{} {}** 🥳",
                        data.amount, data.currency
                    ))
                    .timestamp(data.timestamp)
                    .color(0xffd43b);

                if let Some(message) = data.message {
                    embed = embed.field("Message", truncate(&message, 1024), false);
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
    h.insert("cross-origin-opener-policy", "same-origin".parse().unwrap());
    h.insert(
        "cross-origin-resource-policy",
        "same-origin".parse().unwrap(),
    );
    h.insert("origin-agent-cluster", "?1".parse().unwrap());
    h.insert("referrer-policy", "no-referrer".parse().unwrap());
    h.insert("x-content-type-options", "nosniff".parse().unwrap());
    h.insert("x-frame-options", "DENY".parse().unwrap());
    h.insert("x-xss-protection", "1; mode=block".parse().unwrap());

    response
}

#[tracing::instrument(skip_all)]
pub async fn serve(serenity_http: Arc<serenity::Http>) -> eyre::Result<()> {
    let state = Arc::new(AppState { serenity_http });

    let listener = TcpListener::bind((CONFIG.host.clone(), CONFIG.port)).await?;
    let local_addr = listener.local_addr()?;

    tracing::info!(
        address = format!("http://{}", local_addr),
        "started API server"
    );

    let app = Router::new()
        .route("/", get(routes::ping).head(routes::ping_head))
        .route("/ko-fi", post(routes::kofi_webhook))
        .fallback(routes::not_found)
        .layer(middleware::from_fn(security_middleware))
        .with_state(state);

    axum::serve(listener, app).await?;

    Ok(())
}
