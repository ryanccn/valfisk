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

impl<'a> From<&'a color_eyre::eyre::Error> for ErrorOrPanic<'a> {
    fn from(val: &'a color_eyre::eyre::Error) -> Self {
        ErrorOrPanic::Error(val)
    }
}

impl<'a> From<&'a Option<String>> for ErrorOrPanic<'a> {
    fn from(val: &'a Option<String>) -> Self {
        ErrorOrPanic::Panic(val)
    }
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
    /// Create a new [`ValfiskError`] from an error or a panic string and Poise context.
    #[must_use]
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
        if let Ok(channel_id) = match std::env::var("ERROR_LOGS_CHANNEL") {
            Ok(channel_id_str) => channel_id_str
                .parse::<u64>()
                .map_err(color_eyre::eyre::Error::from),
            Err(err) => Err(color_eyre::eyre::Error::from(err)),
        } {
            let channel = ChannelId::new(channel_id);

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
                .send_message(&self.ctx, CreateMessage::default().embed(embed))
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
