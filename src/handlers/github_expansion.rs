use poise::serenity_prelude as serenity;

use crate::reqwest_client;
use regex::Regex;

use anyhow::Result;
use once_cell::sync::Lazy;

static GITHUB: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w-]+/[\w.-]+)/blob/(?P<ref>.+?)/(?P<file>.*)#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    let message = message.clone();
    let mut embeds: Vec<serenity::CreateEmbed> = vec![];

    for captures in GITHUB.captures_iter(&message.content) {
        let repo = captures["repo"].to_owned();
        let ref_ = captures["ref"].to_owned();
        let file = captures["file"].to_owned();

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
            .map(|s| s.to_owned())
            .collect();

        let idx_start = start - 1;
        let idx_end = match end {
            Some(end) => end,
            None => start,
        };
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
                    .allowed_mentions(serenity::CreateAllowedMentions::new().replied_user(false))
                    .reference_message(&message),
            )
            .await?;
    };

    Ok(())
}
