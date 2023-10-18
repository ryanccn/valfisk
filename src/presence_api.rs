use poise::serenity_prelude as serenity;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde_json::json;

use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;

use owo_colors::OwoColorize;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
    HttpResponse::Ok().json(json!({"ok": true}))
}

#[get("/presence/{user}")]
async fn route_get_presence(path: web::Path<(u64,)>) -> impl Responder {
    let path = path.into_inner();
    let user_id = serenity::UserId::new(path.0);

    let store = PRESENCE_STORE.lock().await;
    let presence_data = store.get(&user_id).cloned();
    drop(store);

    if let Some(presence_data) = presence_data {
        HttpResponse::Ok().json(presence_data)
    } else {
        HttpResponse::NotFound().json(json!({"error": "User not found!"}))
    }
}

pub async fn serve() -> Result<()> {
    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_owned());
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse::<u16>()?;

    println!(
        "{} API server {}",
        "Started".green(),
        format!("http://{}:{}", host, port).dimmed()
    );

    HttpServer::new(|| App::new().service(route_get_presence))
        .bind((host, port))?
        .run()
        .await?;

    Ok(())
}
