use once_cell::sync::Lazy;
use std::fmt::Debug;

use nanoid::nanoid;
use poise::{
    serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Timestamp},
    CreateReply,
};

use crate::Context;
use tracing::error;

/// A wrapper type that encapsulates errors ([`color_eyre::eyre::Error`]) or panic strings ([`Option<String>`]).
pub enum ErrorOrPanic<'a> {
    /// A reference to an error, [`color_eyre::eyre::Error`]
    Error(&'a color_eyre::eyre::Error),
    /// A reference to a panic string, [`Option<String>`]
    Panic(&'a Option<String>),
}

impl ErrorOrPanic<'_> {
    /// Return whether `self` is a panic or an error.
    fn type_string(&self) -> String {
        match self {
            Self::Panic(_) => "panic".to_owned(),
            Self::Error(_) => "error".to_owned(),
        }
    }
}

pub static ERROR_LOGS_CHANNEL: Lazy<Option<ChannelId>> = Lazy::new(|| {
    std::env::var("ERROR_LOGS_CHANNEL")
        .ok()
        .and_then(|s| s.parse::<ChannelId>().ok())
});

/// A wrapped type around errors or panics encapsulated in [`ErrorOrPanic`] that includes context from Poise and a randomly generated `error_id`.
#[derive(Debug)]
pub struct ValfiskError<'a> {
    /// The error or panic.
    pub error_or_panic: ErrorOrPanic<'a>,
    /// The Poise context.
    pub ctx: &'a Context<'a>,
    /// A randomly generated error ID.
    pub error_id: String,
}

impl ValfiskError<'_> {
    /// Create a new [`ValfiskError`] from an error and Poise context.
    #[must_use]
    pub fn error<'a>(error: &'a color_eyre::eyre::Error, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            error_or_panic: ErrorOrPanic::Error(error),
            ctx,
            error_id: nanoid!(8),
        }
    }

    /// Create a new [`ValfiskError`] from a panic string and Poise context.
    #[must_use]
    pub fn panic<'a>(panic: &'a Option<String>, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            error_or_panic: ErrorOrPanic::Panic(panic),
            ctx,
            error_id: nanoid!(8),
        }
    }

    /// Log the error to the console.
    #[tracing::instrument(skip(self), fields(id = self.error_id, r#type = self.error_or_panic.type_string(), command = self.ctx.invocation_string(), channel = self.ctx.channel_id().get(), author = self.ctx.author().id.get()))]
    pub fn handle_log(&self) {
        error!("{:#?}", self.error_or_panic);
    }

    /// Reply to the interaction with an embed informing the user of an error, containing the randomly generated error ID.
    #[tracing::instrument(skip(self))]
    pub async fn handle_reply(&self) {
        self.ctx
            .send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .title("An error occurred!")
                        .description("Hmm. I wonder what happened there?")
                        .footer(CreateEmbedFooter::new(&self.error_id))
                        .timestamp(Timestamp::now())
                        .color(0xff6b6b),
                ),
            )
            .await
            .ok();
    }

    /// Report the error to a channel defined through the environment variable `ERROR_LOGS_CHANNEL`.
    #[tracing::instrument(skip(self))]
    pub async fn handle_report(&self) {
        if let Some(channel) = *ERROR_LOGS_CHANNEL {
            let embed = CreateEmbed::default()
                .title("An error occurred!")
                .description(format!("```\n{:#?}\n```", self.error_or_panic))
                .footer(CreateEmbedFooter::new(&self.error_id))
                .timestamp(Timestamp::now())
                .color(0xff6b6b)
                .field(
                    "Command",
                    format!("`{}`", self.ctx.invocation_string()),
                    false,
                )
                .field(
                    "Channel",
                    format!("<#{}>", self.ctx.channel_id().get()),
                    false,
                )
                .field("User", format!("<@{}>", self.ctx.author().id.get()), false);

            channel
                .send_message(self.ctx.http(), CreateMessage::default().embed(embed))
                .await
                .ok();
        }
    }

    /// Log the error to the console, send an error reply to the interaction, and report the error in the error channel.
    #[tracing::instrument(skip(self))]
    pub async fn handle_all(&self) {
        self.handle_log();
        self.handle_reply().await;
        self.handle_report().await;
    }
}

impl Debug for ErrorOrPanic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(e) => Debug::fmt(e, f),
            Self::Panic(p) => Debug::fmt(p, f),
        }
    }
}
