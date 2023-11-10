use nanoid::nanoid;
use owo_colors::OwoColorize;
use poise::{
    serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Timestamp},
    CreateReply,
};

use crate::Context;

/// A wrapper type that encapsulates errors ([`anyhow::Error`]) or panic strings ([`Option<String>`]).
pub enum ErrorOrPanic<'a> {
    /// A reference to an error, [`anyhow::Error`]
    Error(&'a anyhow::Error),
    /// A reference to a panic string, [`Option<String>`]
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
    /// Return whether `self` is a panic or an error.
    fn type_string(&self) -> String {
        match self {
            Self::Panic(_) => "panic".to_owned(),
            Self::Error(_) => "error".to_owned(),
        }
    }
}

/// A wrapped type around errors or panics encapsulated in [`ErrorOrPanic`] that includes context from Poise and a randomly generated `error_id`.
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
    pub fn handle_log(&self) {
        eprintln!(
            "{}\n  {} {}\n  {} {}\n{:#?}",
            format!("Encountered {}!", self.error_or_panic.type_string()).red(),
            "ID:".dimmed(),
            self.error_id,
            "Command:".dimmed(),
            self.ctx.invoked_command_name(),
            self.error_or_panic
        );
    }

    /// Reply to the interaction with an embed informing the user of an error, containing the randomly generated error ID.
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

    /// Report the error to a channel defined through the environment variable `ERROR_LOGS_CHANNEL`.
    pub async fn handle_report(&self) {
        if let Ok(channel_id) = match std::env::var("ERROR_LOGS_CHANNEL") {
            Ok(channel_id_str) => channel_id_str.parse::<u64>().map_err(anyhow::Error::from),
            Err(err) => Err(anyhow::Error::from(err)),
        } {
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

    /// Log the error to the console, send an error reply to the interaction, and report the error in the error channel.
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
