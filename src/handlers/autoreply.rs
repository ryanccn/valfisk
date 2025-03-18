// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;
use rand::seq::IndexedRandom as _;
use regex::RegexBuilder;

use crate::config::CONFIG;

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.guild_id != CONFIG.guild_id {
        return Ok(());
    }

    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if let Some(storage) = &ctx.data::<crate::Data>().storage {
        let data = storage
            .getall_autoreply()
            .await?
            .into_iter()
            .map(|(k, v)| {
                RegexBuilder::new(&k)
                    .multi_line(true)
                    .build()
                    .map(|r| (r, v))
                    .map_err(|e| e.into())
            })
            .collect::<Result<Vec<_>>>()?;

        let responses = data
            .iter()
            .flat_map(|(regex, template)| {
                regex.captures_iter(&message.content).map(|captures| {
                    let mut expanded = String::new();
                    captures.expand(template, &mut expanded);
                    expanded
                })
            })
            .collect::<Vec<_>>();

        let possible_reply = {
            let mut rng = rand::rng();
            responses.choose(&mut rng)
        };

        if let Some(reply) = possible_reply {
            if !reply.is_empty() {
                message.reply(&ctx.http, reply).await?;
            }
        }
    }

    Ok(())
}
