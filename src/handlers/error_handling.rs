// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use tracing::error;

use poise::{
    serenity_prelude::{CacheHttp, CreateEmbed, CreateMessage, Timestamp},
    FrameworkError,
};

use crate::{
    utils::error_handling::{ValfiskError, ERROR_LOGS_CHANNEL},
    Data,
};

#[tracing::instrument(skip(err))]
pub async fn handle_error(err: FrameworkError<'_, Data, eyre::Report>) {
    match err {
        FrameworkError::EventHandler {
            error,
            event,
            framework,
            ..
        } => {
            error!("{:?}", error);

            let embed = CreateEmbed::default()
                .title("An error occurred in an event handler!")
                .description(format!(
                    "### Error\n```\n{error:#?}\n```\n### Event\n```\n{event:#?}\n```"
                ))
                .timestamp(Timestamp::now())
                .color(0xff6b6b);

            if let Some(channel) = *ERROR_LOGS_CHANNEL {
                channel
                    .send_message(
                        framework.serenity_context.http(),
                        CreateMessage::default().embed(embed),
                    )
                    .await
                    .ok();
            }
        }

        FrameworkError::Command { error, ctx, .. } => {
            ValfiskError::error(&error, &ctx).handle_all().await;
        }

        FrameworkError::CommandPanic { payload, ctx, .. } => {
            ValfiskError::panic(&payload, &ctx).handle_all().await;
        }

        _ => {}
    }
}
