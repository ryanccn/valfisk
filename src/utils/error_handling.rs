use nanoid::nanoid;
use owo_colors::OwoColorize;
use poise::{
    serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Timestamp},
    CreateReply,
};

use crate::Context;

pub enum ErrorOrPanic<'a> {
    Error(&'a anyhow::Error),
    Panic(&'a Option<String>),
}

impl<'a> From<&'a anyhow::Error> for ErrorOrPanic<'a> {
    fn from(val: &'a anyhow::Error) -> Self {
        ErrorOrPanic::Error(val)
    }
}

impl<'a> From<&'a Option<String>> for ErrorOrPanic<'a> {
    fn from(val: &'a Option<String>) -> Self {
        ErrorOrPanic::Panic(val)
    }
}

impl ErrorOrPanic<'_> {
    fn type_(&self) -> String {
        match self {
            Self::Panic(_) => "panic".to_owned(),
            Self::Error(_) => "error".to_owned(),
        }
    }
}

pub struct ValfiskError<'a> {
    pub error_or_panic: ErrorOrPanic<'a>,
    pub ctx: &'a Context<'a>,
    pub error_id: String,
}

impl ValfiskError<'_> {
    pub fn new<'a>(
        error_or_panic: impl Into<ErrorOrPanic<'a>>,
        ctx: &'a Context,
    ) -> ValfiskError<'a> {
        ValfiskError {
            error_or_panic: error_or_panic.into(),
            ctx,
            error_id: nanoid!(8),
        }
    }

    pub fn handle_log(&self) {
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

    pub async fn handle_reply(&self) {
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

    pub async fn handle_report(&self) {
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
                .color(0xef4444)
                .field(
                    "Command",
                    format!("`{}`", self.ctx.invoked_command_name()),
                    false,
                )
                .field(
                    "Channel",
                    format!("<#{}>", self.ctx.channel_id().get()),
                    false,
                )
                .field("User", format!("<@{}>", self.ctx.author().id.get()), false);

            channel
                .send_message(&self.ctx, CreateMessage::new().embed(embed))
                .await
                .ok();
        }
    }

    pub async fn handle_all(&self) {
        self.handle_log();
        self.handle_reply().await;
        self.handle_report().await;
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
