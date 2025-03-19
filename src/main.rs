// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Report, Result};
use std::{process::ExitCode, sync::Arc};
use tokio::{signal, sync::mpsc, task};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use poise::{Framework, FrameworkOptions, serenity_prelude as serenity};

use crate::{
    config::CONFIG, event_handler::EventHandler, safe_browsing::SafeBrowsing, storage::Storage,
    utils::ExitCodeError,
};

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
        let storage = if let Some(url) = &CONFIG.redis_url {
            let client = redis::Client::open(url.clone())?;
            Some(Storage::from(client))
        } else {
            tracing::warn!("REDIS_URL is not configured, some features may be disabled");
            None
        };

        let safe_browsing = if let Some(key) = &CONFIG.safe_browsing_api_key {
            Some(SafeBrowsing::new(key))
        } else {
            tracing::warn!(
                "SAFE_BROWSING_API_KEY is not configured, Safe Browsing will be disabled"
            );
            None
        };

        Ok(Self {
            storage,
            safe_browsing,
        })
    }
}

pub type Context<'a> = poise::Context<'a, Data, Report>;

async fn shutdown() {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(10);

    task::spawn({
        let shutdown_tx = shutdown_tx.clone();
        async move {
            if signal::ctrl_c().await.is_ok() {
                let _ = shutdown_tx.send(()).await;
            }
        }
    });

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        task::spawn({
            let shutdown_tx = shutdown_tx.clone();
            async move {
                if let Ok(mut sigterm_signal) = signal(SignalKind::terminate()) {
                    if sigterm_signal.recv().await.is_some() {
                        let _ = shutdown_tx.send(()).await;
                    }
                }
            }
        });
    }

    shutdown_rx.recv().await;
}

async fn valfisk() -> Result<()> {
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

    let data = Arc::new(Data::new()?);

    if let Some(safe_browsing) = &data.safe_browsing {
        safe_browsing.update().await?;
    }

    let mut client = serenity::Client::builder(
        CONFIG.discord_token.parse()?,
        serenity::GatewayIntents::non_privileged()
            .union(serenity::GatewayIntents::GUILD_MEMBERS)
            .union(serenity::GatewayIntents::MESSAGE_CONTENT),
    )
    .event_handler(EventHandler)
    .framework(Framework::new(FrameworkOptions {
        commands: commands::to_vec(),
        on_error: |err| Box::pin(handlers::error(err)),
        ..Default::default()
    }))
    .data(data.clone())
    .await?;

    tokio::select! {
        () = shutdown() => {
            tracing::warn!("Shutdown signal received, exiting!");
            Err(ExitCodeError(1).into())
        },

        result = api::serve(client.http.clone()) => { result },
        result = schedule::run(client.http.clone(), data.clone()) => { result },
        result = client.start() => { result.map_err(|e| e.into()) },
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = valfisk().await {
        if let Some(exit_code) = err.downcast_ref::<ExitCodeError>() {
            exit_code.as_std()
        } else {
            eprintln!("Error: {err:?}");
            ExitCode::FAILURE
        }
    } else {
        ExitCode::SUCCESS
    }
}
