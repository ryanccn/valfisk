// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Report, Result};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use poise::{Framework, FrameworkOptions, serenity_prelude as serenity};

use crate::config::CONFIG;
use crate::event_handler::EventHandler;
use crate::safe_browsing::SafeBrowsing;
use crate::storage::Storage;

mod api;
mod commands;
mod config;
mod event_handler;
mod handlers;
mod http;
mod intelligence;
mod safe_browsing;
mod schedule;
mod storage;
mod template_channel;
mod utils;

#[derive(Debug)]
pub struct Data {
    storage: Option<Storage>,
    safe_browsing: Option<SafeBrowsing>,
}

impl Data {
    fn new() -> Result<Self> {
        let storage = if let Some(redis_url) = &CONFIG.redis_url {
            let client = redis::Client::open(redis_url.clone())?;
            Some(Storage::from(client))
        } else {
            None
        };

        let safe_browsing = CONFIG
            .safe_browsing_api_key
            .as_ref()
            .map(|key| SafeBrowsing::new(key));

        Ok(Self {
            storage,
            safe_browsing,
        })
    }
}

pub type Context<'a> = poise::Context<'a, Data, Report>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("valfisk=info,warn,error")),
        )
        .with(tracing_error::ErrorLayer::default())
        .init();

    #[cfg(debug_assertions)]
    {
        if let Ok(dotenv_path) = dotenvy::dotenv() {
            tracing::warn!(
                "Loaded environment variables from {}",
                dotenv_path.display()
            );
        }
    }

    // Preload config from environment
    let _ = *CONFIG;

    if CONFIG.redis_url.is_none() {
        tracing::warn!("`REDIS_URL` is not configured, some features may be disabled");
    };

    let data = Arc::new(Data::new()?);

    if let Some(safe_browsing) = &data.safe_browsing {
        safe_browsing.update().await?;
    }

    let mut client = serenity::Client::builder(
        CONFIG.discord_token.parse()?,
        serenity::GatewayIntents::all(),
    )
    .event_handler(EventHandler)
    .framework(Framework::new(FrameworkOptions {
        commands: commands::to_vec(),
        on_error: |err| Box::pin(handlers::handle_error(err)),
        ..Default::default()
    }))
    .data(data.clone())
    .await?;

    tokio::select! {
        result = api::serve(client.http.clone()) => { result },
        result = schedule::start(client.http.clone(), data.clone()) => { result },
        result = client.start() => { result.map_err(eyre::Report::from) },
        _ = tokio::signal::ctrl_c() => {
            tracing::warn!("Interrupted with Ctrl-C, exiting");
            std::process::exit(1);
        },
    }
}
