// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude as serenity;

use axum::{
    extract::Request,
    middleware,
    response::Response,
    routing::{Router, get},
};

use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::CONFIG;

mod routes {
    use axum::{
        extract::Query,
        http::StatusCode,
        response::{IntoResponse, Json},
    };
    use serde_json::json;

    use crate::utils::AxumResult;

    #[tracing::instrument(skip_all)]
    pub async fn ping() -> impl IntoResponse {
        (StatusCode::OK, Json(json!({ "ok": true })))
    }

    #[tracing::instrument(skip_all)]
    pub async fn ping_head() -> impl IntoResponse {
        StatusCode::OK
    }

    #[derive(serde::Deserialize)]
    pub struct ExchangeQuery {
        from: String,
        to: String,
        amount: Option<f64>,
    }

    #[tracing::instrument(skip_all)]
    pub async fn exchange(Query(query): Query<ExchangeQuery>) -> AxumResult<impl IntoResponse> {
        use crate::commands::exchange::{
            fetch_frankfurter, fetch_mastercard, fetch_revolut, fetch_visa, fetch_wise,
        };

        let ExchangeQuery { from, to, amount } = query;
        let amount = amount.unwrap_or(1.);

        let (frankfurter, wise, revolut, visa, mastercard) = tokio::join!(
            fetch_frankfurter(&from, &to),
            fetch_wise(&from, &to, amount),
            fetch_revolut(&from, &to, amount),
            fetch_visa(&from, &to, amount),
            fetch_mastercard(&from, &to, amount),
        );

        Ok((
            StatusCode::OK,
            Json(json!({
                "frankfurter": frankfurter.map_or_else(|_| serde_json::Value::Null, |rate| json!({
                    "amount": amount * rate,
                    "rate": rate,
                })),
                "wise": wise.map_or_else(|_| Ok(serde_json::Value::Null), serde_json::to_value)?,
                "revolut": revolut.map_or_else(|_| Ok(serde_json::Value::Null), serde_json::to_value)?,
                "visa": visa.map_or_else(|_| Ok(serde_json::Value::Null), serde_json::to_value)?,
                "mastercard": mastercard.map_or_else(|_| Ok(serde_json::Value::Null), serde_json::to_value)?,
            })),
        ))
    }

    #[tracing::instrument(skip_all)]
    pub async fn not_found() -> impl IntoResponse {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Not found" })))
    }
}

#[derive(Debug)]
#[expect(dead_code)]
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
        .route("/exchange", get(routes::exchange))
        .fallback(routes::not_found)
        .layer(middleware::from_fn(security_middleware))
        .with_state(state);

    axum::serve(listener, app).await?;

    Ok(())
}
