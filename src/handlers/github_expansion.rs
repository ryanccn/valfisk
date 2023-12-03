use std::time::Duration;

use poise::serenity_prelude::{self as serenity, futures::StreamExt, EditMessage, Event};
use tokio::time::timeout;

use crate::reqwest_client;
use regex::Regex;

use color_eyre::eyre::Result;
use log::debug;
use once_cell::sync::Lazy;

static GITHUB: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w-]+/[\w.-]+)/blob/(?P<ref>.+?)/(?P<file>.*)#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    let mut embeds: Vec<serenity::CreateEmbed> = vec![];

    for captures in GITHUB.captures_iter(&message.content) {
        debug!(
            "Handling GitHub link {} on message {}",
            &captures[0], message.id
        );

        let repo = &captures["repo"];
        let ref_ = &captures["ref"];
        let file = &captures["file"];

        let language = file.split('.').last().unwrap_or("").to_owned();

        let start = captures["start"].parse::<usize>()?;
        let end = captures
            .name("end")
            .and_then(|end| end.as_str().parse::<usize>().ok());

        let resp = reqwest_client::HTTP
            .get(format!(
                "https://raw.githubusercontent.com/{repo}/{ref_}/{file}"
            ))
            .send()
            .await?;

        let lines: Vec<String> = resp
            .text()
            .await?
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();

        let idx_start = start - 1;
        let idx_end = end.unwrap_or(start);

        let selected_lines = &lines[idx_start..idx_end];

        let embed = serenity::CreateEmbed::new()
            .title(repo)
            .field(
                file,
                "```".to_owned() + &language + "\n" + &selected_lines.join("\n") + "\n```",
                true,
            )
            .footer(serenity::CreateEmbedFooter::new(ref_))
            .timestamp(serenity::Timestamp::now());

        embeds.push(embed);
    }

    if !embeds.is_empty() {
        let msg_id = message.id;

        let mut message_updates = serenity::collector::collect(&ctx.shard, move |ev| match ev {
            Event::MessageUpdate(x) if x.id == msg_id => Some(()),
            _ => None,
        });

        let _ = timeout(Duration::from_millis(2000), message_updates.next()).await;

        ctx.http
            .edit_message(
                message.channel_id,
                message.id,
                &EditMessage::new().suppress_embeds(true),
                vec![],
            )
            .await?;

        message
            .channel_id
            .send_message(
                &ctx,
                serenity::CreateMessage::new()
                    .embeds(embeds)
                    .allowed_mentions(serenity::CreateAllowedMentions::new().replied_user(false))
                    .reference_message(message),
            )
            .await?;
    };

    Ok(())
}
