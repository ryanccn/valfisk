use poise::serenity_prelude as serenity;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use serde_json::json;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::utils::actix_utils::ActixError;
use owo_colors::OwoColorize;

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

pub static PRESENCE_STORE: Lazy<Mutex<HashMap<serenity::UserId, ValfiskPresenceData>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[get("/")]
async fn route_ping() -> impl Responder {
    HttpResponse::Ok().json(json!({ "ok": true }))
}

#[get("/presence/{user}")]
async fn route_get_presence(path: web::Path<(u64,)>) -> Result<impl Responder, ActixError> {
    let path = path.into_inner();
    if path.0 == 0 {
        return Ok(HttpResponse::BadRequest().json(json!({ "error": "User ID cannot be 0!" })));
    }

    let user_id = serenity::UserId::from(path.0);

    let store = PRESENCE_STORE.lock().unwrap();
    let presence_data = store.get(&user_id).cloned();
    drop(store);

    match presence_data {
        Some(presence_data) => Ok(HttpResponse::Ok().json(presence_data)),
        None => Ok(HttpResponse::NotFound().json(json!({ "error": "User not found!" }))),
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let host = std::env::var("HOST").unwrap_or(match cfg!(debug_assertions) {
        true => "127.0.0.1".to_owned(),
        false => "0.0.0.0".to_owned(),
    });
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse::<u16>()?;

    println!(
        "{} API server {}",
        "Started".green(),
        format!("http://{}:{}", host, port).dimmed()
    );

    HttpServer::new(|| {
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
            .add(("x-xss-protection", "0"));

        App::new()
            .wrap(security_middleware)
            .service(route_ping)
            .service(route_get_presence)
    })
    .bind((host, port))?
    .run()
    .await?;

    Ok(())
}
