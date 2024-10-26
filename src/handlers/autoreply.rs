// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;
use rand::seq::SliceRandom as _;

use crate::{config::CONFIG, Data};

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(
    ctx: &serenity::Context,
    data: &Data,
    message: &serenity::Message,
) -> Result<()> {
    if message.guild_id != CONFIG.guild_id {
        return Ok(());
    }

    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if let Some(storage) = &data.storage {
        let autoreply_data = storage.getall_autoreply().await?;
        let responses: Vec<String> = autoreply_data
            .into_iter()
            .filter(|(keyword, _)| {
                message
                    .content
                    .to_lowercase()
                    .contains(&keyword.to_lowercase())
            })
            .map(|(_, response)| response)
            .collect();

        let possible_reply = {
            let mut rng = rand::thread_rng();
            responses.choose(&mut rng)
        };

        if let Some(reply) = possible_reply {
            message.reply(&ctx.http, reply).await?;
        }
    }

    Ok(())
}
