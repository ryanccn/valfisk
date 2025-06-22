// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt;

use poise::{
    CreateReply,
    serenity_prelude::{CreateEmbed, CreateEmbedFooter, CreateMessage, Timestamp},
};

use crate::{Context, config::CONFIG, utils::nanoid};

use super::serenity::format_mentionable;

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
    report_or_panic: ReportOrPanic<'a>,
    /// The Poise context.
    ctx: &'a Context<'a>,
    /// A randomly generated error ID.
    error_id: String,
}

impl ValfiskError<'_> {
    /// Create a new [`ValfiskError`] from a report and Poise context.
    #[must_use]
    pub fn error<'a>(error: &'a eyre::Report, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            report_or_panic: ReportOrPanic::Report(error),
            ctx,
            error_id: nanoid(8),
        }
    }

    /// Create a new [`ValfiskError`] from a panic string and Poise context.
    #[must_use]
    pub fn panic<'a>(panic: Option<&'a str>, ctx: &'a Context) -> ValfiskError<'a> {
        ValfiskError {
            report_or_panic: ReportOrPanic::Panic(panic),
            ctx,
            error_id: nanoid(8),
        }
    }

    /// Log the error to the console.
    #[tracing::instrument(skip(self))]
    pub fn handle_log(&self) {
        tracing::error!(
            id = self.error_id,
            command = self.ctx.invocation_string(),
            channel = self.ctx.channel_id().get(),
            author = self.ctx.author().id.get(),
            "{:?}",
            self.report_or_panic,
        );
    }

    /// Reply to the interaction with an embed informing the user of an error, containing the randomly generated error ID.
    #[tracing::instrument(skip(self))]
    pub async fn handle_reply(&self) {
        if let Err(err) = self
            .ctx
            .send(
                CreateReply::default().embed(
                    CreateEmbed::default()
                        .title("An error occurred!")
                        .description("You can contact the owner of this app with the error ID shown below if you need support.")
                        .footer(CreateEmbedFooter::new(&self.error_id))
                        .timestamp(Timestamp::now())
                        .color(0xff6b6b),
                ),
            )
            .await
        {
            tracing::error!("{err:?}");
        }
    }

    /// Report the error to a channel defined through the environment variable `ERROR_LOGS_CHANNEL`.
    #[tracing::instrument(skip(self))]
    pub async fn handle_report(&self) {
        if let Some(channel) = CONFIG.error_logs_channel {
            let mut error_string = format!("{:#?}", self.report_or_panic);
            error_string = error_string.replace(&CONFIG.discord_token, "<redacted>");

            if let Some(data) = &CONFIG.redis_url {
                error_string = error_string.replace(data, "<redacted>");
            }
            if let Some(data) = &CONFIG.pagespeed_api_key {
                error_string = error_string.replace(data, "<redacted>");
            }
            if let Some(data) = &CONFIG.safe_browsing_api_key {
                error_string = error_string.replace(data, "<redacted>");
            }
            if let Some(data) = &CONFIG.translation_api_key {
                error_string = error_string.replace(data, "<redacted>");
            }

            let mut embed = CreateEmbed::default()
                .title("An error occurred!")
                .description(format!("```\n{error_string}\n```"))
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
                    format_mentionable(Some(self.ctx.channel_id())),
                    false,
                )
                .field(
                    "User",
                    format_mentionable(Some(self.ctx.author().id)),
                    false,
                );

            if let Some(guild) = self.ctx.partial_guild().await {
                embed = embed.field(
                    "Guild",
                    format!(
                        "**{}** ({})\n*Owner*: {}",
                        guild.name,
                        guild.id,
                        format_mentionable(Some(guild.owner_id)),
                    ),
                    false,
                );
            }

            if let Err(err) = channel
                .send_message(self.ctx.http(), CreateMessage::default().embed(embed))
                .await
            {
                tracing::error!("{err:?}");
            }
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

impl fmt::Debug for ReportOrPanic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Report(e) => fmt::Debug::fmt(e, f),
            Self::Panic(p) => fmt::Debug::fmt(p, f),
        }
    }
}
