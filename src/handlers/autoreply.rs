// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;
use rand::seq::IndexedRandom as _;
use regex::{Regex, RegexBuilder};

use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

type CompiledCache = HashMap<serenity::GuildId, HashMap<String, Regex>>;

static COMPILED: LazyLock<RwLock<CompiledCache>> =
    LazyLock::new(|| RwLock::new(CompiledCache::new()));

/// Compile a guild's patterns, reusing anything already compiled and dropping cached
/// entries the guild no longer configures. Returns one entry per pattern, in order,
/// with [`None`] for patterns that do not compile.
fn compile(guild: serenity::GuildId, patterns: &[(String, String)]) -> Vec<Option<Regex>> {
    let mut cache = COMPILED.write().unwrap();
    let guild_cache = cache.entry(guild).or_default();

    guild_cache.retain(|cached, _| patterns.iter().any(|(pattern, _)| pattern == cached));

    patterns
        .iter()
        .map(|(pattern, _)| {
            if let Some(regex) = guild_cache.get(pattern) {
                return Some(regex.clone());
            }

            let regex = RegexBuilder::new(pattern).multi_line(true).build().ok()?;
            guild_cache.insert(pattern.clone(), regex.clone());

            Some(regex)
        })
        .collect()
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) -> Result<()> {
    if message.author.id == ctx.cache.current_user().id {
        return Ok(());
    }

    if let Some(guild_id) = message.guild_id
        && let Some(storage) = &ctx.data::<crate::Data>().storage
    {
        let data = storage.scan_autoreply(guild_id).await?;

        let responses = compile(guild_id, &data)
            .into_iter()
            .zip(&data)
            .filter_map(|(regex, (_, replacement))| {
                regex?.captures(&message.content).map(|captures| {
                    let mut expanded = String::new();
                    captures.expand(replacement, &mut expanded);
                    expanded
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns(patterns: &[&str]) -> Vec<(String, String)> {
        patterns
            .iter()
            .map(|p| ((*p).to_owned(), String::new()))
            .collect()
    }

    fn cached(guild: serenity::GuildId) -> Vec<String> {
        let cache = COMPILED.read().unwrap();
        let mut keys = cache[&guild].keys().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }

    #[test]
    fn compile_caches_and_prunes() {
        let guild = serenity::GuildId::new(1);

        let compiled = compile(guild, &patterns(&["foo", "bar"]));
        assert!(compiled.iter().all(Option::is_some));
        assert_eq!(cached(guild), ["bar", "foo"]);

        compile(guild, &patterns(&["foo"]));
        assert_eq!(cached(guild), ["foo"]);
    }

    #[test]
    fn compile_reports_invalid_patterns() {
        let guild = serenity::GuildId::new(2);

        let compiled = compile(guild, &patterns(&["valid", "("]));

        assert!(compiled[0].is_some());
        assert!(compiled[1].is_none());
        assert_eq!(cached(guild), ["valid"]);
    }
}
