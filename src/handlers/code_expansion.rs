use poise::serenity_prelude as serenity;

use crate::{reqwest_client, utils::serenity::suppress_embeds};
use regex::Regex;

use color_eyre::eyre::Result;
use once_cell::sync::Lazy;
use tracing::debug;

static GITHUB: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://github\.com/(?P<repo>[\w-]+/[\w.-]+)/blob/(?P<ref>\S+?)/(?P<file>\S+)#L(?P<start>\d+)(?:[~-]L?(?P<end>\d+)?)?").unwrap()
});

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
async fn github(message: &serenity::Message) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    for captures in GITHUB.captures_iter(&message.content) {
        debug!(
            "Handling GitHub link {} on message {}",
            &captures[0], message.id
        );

        let repo = captures["repo"].to_owned();
        let ref_ = captures["ref"].to_owned();
        let file = captures["file"].to_owned();

        let file_for_language = file.clone();
        let language = file_for_language.split('.').last().unwrap_or("");

        let start = captures["start"].parse::<usize>()?;
        let end = captures
            .name("end")
            .and_then(|end| end.as_str().parse::<usize>().ok());

        let lines: Vec<String> = reqwest_client::HTTP
            .get(format!(
                "https://raw.githubusercontent.com/{repo}/{ref_}/{file}"
            ))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .lines()
            .map(|s| s.to_owned())
            .collect();

        let idx_start = start - 1;
        let idx_end = end.unwrap_or(start);

        let selected_lines = &lines[idx_start..idx_end];

        let embed = serenity::CreateEmbed::default()
            .title(format!(
                "{repo} {file} L{start}{}",
                end.map_or_else(String::new, |end| format!("-{end}"))
            ))
            .description("```".to_owned() + language + "\n" + &selected_lines.join("\n") + "\n```")
            .footer(serenity::CreateEmbedFooter::new(ref_))
            .timestamp(serenity::Timestamp::now());

        embeds.push(embed);
    }

    Ok(embeds)
}

static RUST_PLAYGROUND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https://play\.rust-lang\.org/\S*[?&]gist=(?P<gist>\w+)").unwrap());

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
async fn rust_playground(message: &serenity::Message) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    for captures in RUST_PLAYGROUND.captures_iter(&message.content) {
        debug!(
            "Handling Rust Playground link {} on message {}",
            &captures[0], message.id
        );

        let gist_id = captures["gist"].to_owned();

        let gist = reqwest_client::HTTP
            .get(format!(
                "https://gist.githubusercontent.com/rust-play/{gist_id}/raw/playground.rs"
            ))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let embed = serenity::CreateEmbed::default()
            .title("Rust Playground")
            .description("```rust\n".to_owned() + &gist + "\n```")
            .footer(serenity::CreateEmbedFooter::new(gist_id))
            .timestamp(serenity::Timestamp::now())
            .color(0xdea584);

        embeds.push(embed);
    }

    Ok(embeds)
}

static GO_PLAYGROUND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https://go\.dev/play/p/(?P<id>[\w-]+)").unwrap());

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
async fn go_playground(message: &serenity::Message) -> Result<Vec<serenity::CreateEmbed>> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    for captures in GO_PLAYGROUND.captures_iter(&message.content) {
        debug!(
            "Handling Go Playground link {} on message {}",
            &captures[0], message.id
        );

        let id = captures["id"].to_owned();

        let code = reqwest_client::HTTP
            .get("https://go.dev/_/share")
            .query(&[("id", &id)])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let embed = serenity::CreateEmbed::default()
            .title("Go Playground")
            .description("```go\n".to_owned() + &code + "\n```")
            .footer(serenity::CreateEmbedFooter::new(id))
            .timestamp(serenity::Timestamp::now())
            .color(0x00b7e7);

        embeds.push(embed);
    }

    Ok(embeds)
}

#[tracing::instrument(skip_all, fields(message_id = message.id.get()))]
pub async fn handle(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();

    embeds.extend(github(message).await?);
    embeds.extend(rust_playground(message).await?);
    embeds.extend(go_playground(message).await?);

    if !embeds.is_empty() {
        suppress_embeds(ctx, message).await?;

        message
            .channel_id
            .send_message(
                &ctx.http,
                serenity::CreateMessage::default()
                    .embeds(embeds)
                    .allowed_mentions(
                        serenity::CreateAllowedMentions::default().replied_user(false),
                    )
                    .reference_message(message),
            )
            .await?;
    };

    Ok(())
}
