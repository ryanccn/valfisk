// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Debug;

use nanoid::nanoid;
use poise::{
    CreateReply,
    serenity_prelude::{
        CreateEmbed, CreateEmbedFooter, CreateMessage, Mentionable as _, Timestamp,
    },
};

use crate::{Context, config::CONFIG};

/// A wrapper type that encapsulates reports ([`eyre::Report`]) or panic strings ([`Option<String>`]).
pub enum ReportOrPanic<'a> {
    /// A reference to a report, [`eyre::Report`]
    Report(&'a eyre::Report),
    /// A reference to a panic string, [`Option<String>`]
    Panic(Option<&'a str>),
}

/// A wrapped type around reports or panics that includes context from Poise and a randomly generated `error_id`.
#[derive(Debug)]
pub struct ValfiskError<'a> {
    /// The report or panic.
    pub report_or_panic: ReportOrPanic<'a>,
    /// The Poise context.
    pub ctx: &'a Context<'a>,
    /// A randomly generated error ID.
    pub error_id: String,
}

impl ValfiskError<'_> {
    /// Create a new [`ValfiskError`] from a report and Poise context.
    #[must_use]
    pub fn error<'a>(error: &'a eyre::Report, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            report_or_panic: ReportOrPanic::Report(error),
            ctx,
            error_id: nanoid!(8),
        }
    }

    /// Create a new [`ValfiskError`] from a panic string and Poise context.
    #[must_use]
    pub fn panic<'a>(panic: Option<&'a str>, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            report_or_panic: ReportOrPanic::Panic(panic),
            ctx,
            error_id: nanoid!(8),
        }
    }

    /// Log the error to the console.
    #[tracing::instrument(skip(self))]
    pub fn handle_log(&self) {
        tracing::error!(
            {
                id = self.error_id,
                command = self.ctx.invocation_string(),
                channel = self.ctx.channel_id().get(),
                author = self.ctx.author().id.get()
            },
            "{:?}",
            self.report_or_panic,
        );
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
        if let Some(channel) = CONFIG.error_logs_channel {
            let embed = CreateEmbed::default()
                .title("An error occurred!")
                .description(format!("```\n{:#?}\n```", self.report_or_panic))
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
                    self.ctx.channel_id().mention().to_string(),
                    false,
                )
                .field("User", self.ctx.author().mention().to_string(), false);

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

impl Debug for ReportOrPanic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Report(e) => Debug::fmt(e, f),
            Self::Panic(p) => Debug::fmt(p, f),
        }
    }
}
