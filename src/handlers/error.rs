use nanoid::nanoid;
use owo_colors::OwoColorize;
use poise::{
    serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Timestamp},
    CreateReply, FrameworkError,
};

use crate::{Context, Data};

enum ErrorOrPanic<'a> {
    Error(&'a anyhow::Error),
    Panic(&'a Option<String>),
}

impl ErrorOrPanic<'_> {
    fn type_(&self) -> String {
        match self {
            Self::Panic(_) => "panic".to_owned(),
            Self::Error(_) => "error".to_owned(),
        }
    }
}

struct ValfiskError<'a> {
    error_or_panic: ErrorOrPanic<'a>,
    ctx: &'a Context<'a>,
    error_id: String,
}

impl ValfiskError<'_> {
    fn from_error<'a>(error: &'a anyhow::Error, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            error_or_panic: ErrorOrPanic::Error(&error),
            ctx,
            error_id: nanoid!(8),
        }
    }

    fn from_panic<'a>(panic_payload: &'a Option<String>, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            error_or_panic: ErrorOrPanic::Panic(&panic_payload),
            ctx,
            error_id: nanoid!(8),
        }
    }

    fn log(&self) {
        eprintln!(
            "{}\n  {} {}\n  {} {}\n{:#?}",
            format!("Encountered {}!", self.error_or_panic.type_()).red(),
            "ID:".dimmed(),
            self.error_id,
            "Command:".dimmed(),
            self.ctx.invoked_command_name(),
            self.error_or_panic
        );
    }

    async fn reply(&self) {
        self.ctx
            .send(
                CreateReply::new().embed(
                    CreateEmbed::new()
                        .title("An error occurred!")
                        .description("Hmm. I wonder what happened there?")
                        .footer(CreateEmbedFooter::new(&self.error_id))
                        .timestamp(Timestamp::now())
                        .color(0xef4444),
                ),
            )
            .await
            .ok();
    }

    async fn post(&self) {
        let channel_id = match std::env::var("ERROR_LOGS_CHANNEL") {
            Ok(channel_id_str) => Some(channel_id_str.parse::<u64>()),
            Err(_) => None,
        };

        if let Some(Ok(channel_id)) = channel_id {
            let channel = ChannelId::new(channel_id);

            let embed = CreateEmbed::new()
                .title("An error occurred!")
                .description(format!("```\n{:#?}\n```", self.error_or_panic))
                .footer(CreateEmbedFooter::new(&self.error_id))
                .timestamp(Timestamp::now())
                .color(0xef4444);

            channel
                .send_message(&self.ctx, CreateMessage::new().embed(embed))
                .await
                .ok();
        }
    }

    async fn handle_all(&self) {
        self.log();
        self.reply().await;
        self.post().await;
    }
}

impl std::fmt::Debug for ErrorOrPanic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(e) => e.fmt(f),
            Self::Panic(p) => p.fmt(f),
        }
    }
}

pub async fn handle_error(err: &FrameworkError<'_, Data, anyhow::Error>) {
    match err {
        FrameworkError::Setup { error, .. } => {
            eprintln!(
                "{} setting up client:\n  {}",
                "Encountered error".red(),
                error
            );
        }

        FrameworkError::EventHandler { error, .. } => {
            eprintln!(
                "{} handling event!\n{:#?}",
                "Encountered error".red(),
                error
            );
        }

        FrameworkError::Command { error, ctx, .. } => {
            ValfiskError::from_error(error, ctx).handle_all().await;
        }

        FrameworkError::CommandPanic { payload, ctx, .. } => {
            ValfiskError::from_panic(payload, ctx).handle_all().await;
        }

        _ => {}
    }
}
