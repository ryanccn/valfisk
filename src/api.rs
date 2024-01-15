use poise::serenity_prelude as serenity;

use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use serde_json::json;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::utils::actix_utils::ActixError;
use log::info;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ValfiskPresenceData {
    pub status: serenity::OnlineStatus,
    pub client_status: Option<serenity::ClientStatus>,
    #[serde(default)]
    pub activities: Vec<serenity::Activity>,
}

impl ValfiskPresenceData {
    pub fn from_presence(presence: &serenity::Presence) -> ValfiskPresenceData {
        Self {
            status: presence.status,
            client_status: presence.client_status.clone(),
            activities: presence.activities.clone(),
        }
    }
}

pub static PRESENCE_STORE: Lazy<RwLock<HashMap<serenity::UserId, ValfiskPresenceData>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[get("/")]
async fn route_ping() -> Result<impl Responder, ActixError> {
    Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}

#[get("/presence/{user}")]
async fn route_get_presence(path: web::Path<(u64,)>) -> Result<impl Responder, ActixError> {
    let path = path.into_inner();
    if path.0 == 0 {
        return Ok(HttpResponse::BadRequest().json(json!({ "error": "User ID cannot be 0!" })));
    }

    let user_id = serenity::UserId::from(path.0);

    let store = PRESENCE_STORE.read().unwrap();
    let presence_data = store.get(&user_id).cloned();
    drop(store);

    match presence_data {
        Some(presence_data) => Ok(HttpResponse::Ok().json(presence_data)),
        None => Ok(HttpResponse::NotFound().json(json!({ "error": "User not found!" }))),
    }
}

#[derive(serde::Deserialize)]
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

#[post("/ko-fi")]
async fn route_kofi_webhook(
    app_data: web::Data<AppState>,
    form: web::Form<KofiFormData>,
) -> Result<impl Responder, ActixError> {
    let data: KofiData = serde_json::from_str(&form.0.data)?;
    let verification_token = std::env::var("KOFI_VERIFICATION_TOKEN")?;

    if data.verification_token != verification_token {
        return Ok(HttpResponse::Unauthorized().json(json!({ "error": "unauthorized" })));
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
                .color(0xfcd34d);

            if let Some(message) = data.message {
                embed = embed.field("Message", message, false);
            }

            channel
                .send_message(
                    &app_data.into_inner(),
                    serenity::CreateMessage::default().embed(embed),
                )
                .await?;
        }
    }

    Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}

struct AppState {
    serenity_http: Arc<serenity::Http>,
}

impl poise::serenity_prelude::CacheHttp for AppState {
    fn http(&self) -> &serenity::Http {
        &self.serenity_http
    }
}

pub async fn serve(serenity_http: Arc<serenity::Http>) -> color_eyre::eyre::Result<()> {
    #[cfg(debug_assertions)]
    let default_host = "127.0.0.1";
    #[cfg(not(debug_assertions))]
    let default_host = "0.0.0.0";
    let host = std::env::var("HOST").unwrap_or_else(|_| default_host.to_owned());
    let port = std::env::var("PORT").map_or(Ok(8080), |v| v.parse::<u16>())?;

    let app_state = web::Data::new(AppState { serenity_http });

    info!("Started API server {}", format!("http://{host}:{port}"));

    HttpServer::new(move || {
        let security_middleware = middleware::DefaultHeaders::new()
            .add(("access-control-allow-origin", "*"))
            .add(("cross-origin-opener-policy", "same-origin"))
            .add(("cross-origin-resource-policy", "same-origin"))
            .add(("origin-agent-cluster", "?1"))
            .add(("referrer-policy", "no-referrer"))
            .add(("x-content-type-options", "nosniff"))
            .add(("x-dns-prefetch-control", "off"))
            .add(("x-download-options", "noopen"))
            .add(("x-frame-options", "SAMEORIGIN"))
            .add(("x-permitted-cross-domain-policies", "none"))
            .add(("x-xss-protection", "1; mode=block"));

        App::new()
            .wrap(security_middleware)
            .app_data(app_state.clone())
            .service(route_ping)
            .service(route_get_presence)
            .service(route_kofi_webhook)
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
