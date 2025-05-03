// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;
use rand::seq::IndexedRandom as _;
use regex::RegexBuilder;

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if let Some(guild_id) = message.guild_id {
        if let Some(storage) = &ctx.data::<crate::Data>().storage {
            let data = storage.scan_autoreply(guild_id).await?;

            let responses = data
                .iter()
                .filter_map(|(pattern, replacement)| {
                    RegexBuilder::new(pattern)
                        .multi_line(true)
                        .build()
                        .ok()
                        .and_then(|regex| {
                            regex.captures(&message.content).map(|captures| {
                                let mut expanded = String::new();
                                captures.expand(replacement, &mut expanded);
                                expanded
                            })
                        })
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();

            let possible_reply = {
                let mut rng = rand::rng();
                responses.choose(&mut rng)
            };

            if let Some(reply) = possible_reply {
                message.reply(&ctx.http, reply).await?;
            }
        }
    }

    Ok(())
}
