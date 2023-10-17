use poise::serenity_prelude as serenity;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::reqwest_client;

static GITHUB: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w-]+/[\w.-]+)/blob/(?P<ref>.+?)/(?P<file>.*\.(?:(?P<language>\w+))?)#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    let message = message.clone();
    let mut embeds: Vec<serenity::CreateEmbed> = vec![];

    for captures in GITHUB.captures_iter(&message.content) {
        let repo = captures
            .name("repo")
            .ok_or_else(|| anyhow!("Could not obtain `repo`"))?
            .as_str();
        let ref_ = captures
            .name("ref")
            .ok_or_else(|| anyhow!("Could not obtain `ref`"))?
            .as_str();
        let file = captures
            .name("file")
            .ok_or_else(|| anyhow!("Could not obtain `file`"))?
            .as_str();

        let language = if let Some(m) = captures.name("language") {
            m.as_str().to_owned()
        } else {
            "".to_owned()
        };

        let start = captures
            .name("start")
            .ok_or_else(|| anyhow!("Could not obtain `start`"))?
            .as_str()
            .parse::<usize>()?;
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
            .split("\n")
            .map(|s| s.to_owned())
            .collect();

        let idx_start = start - 1;
        let idx_end = if let Some(end) = end {
            end
        } else {
            lines.len()
        };
        let selected_lines = &lines[idx_start..idx_end];

        let embed = serenity::CreateEmbed::new()
            .title(repo)
            .field(
                file,
                "```".to_owned() + &language + "\n" + &selected_lines.join("\n") + "\n```",
                true,
            )
            .footer(serenity::CreateEmbedFooter::new(ref_));

        embeds.push(embed);
    }

    // {
    //     let msg_id = message.id;
    //     let mut message_updates = serenity::collector::collect(&ctx.shard, move |ev| match ev {
    //         serenity::Event::MessageUpdate(x) if x.id == msg_id => Some(()),
    //         _ => None,
    //     });
    //     let _ = tokio::time::timeout(Duration::from_millis(2000), message_updates.next()).await;
    //     message
    //         .edit(&ctx, serenity::EditMessage::new().suppress_embeds(true))
    //         .await?;
    // }

    if !embeds.is_empty() {
        message
            .channel_id
            .send_message(
                &ctx,
                serenity::CreateMessage::new()
                    .embeds(embeds)
                    .reference_message(&message),
            )
            .await?;
    };

    Ok(())
}
