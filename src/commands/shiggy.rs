use poise::{serenity_prelude as serenity, CreateReply};

use crate::{utils::error_handling::ValfiskError, Context};
use color_eyre::eyre::Result;

#[derive(serde::Deserialize)]
struct SafebooruResponse {
    id: i64,
    source: String,
    tag_string: String,
    file_url: String,
}

/// Fetch a random shiggy
#[poise::command(slash_command, guild_only)]
pub async fn shiggy(
    ctx: Context<'_>,

    #[description = "Whether to send the image URL only"]
    #[flag]
    raw: bool,
) -> Result<()> {
    ctx.defer().await?;
    let mut url = "https://safebooru.donmai.us/posts/random.json".parse::<reqwest::Url>()?;
    url.query_pairs_mut()
        .append_pair("tags", "kemomimi-chan_(naga_u) naga_u")
        .append_pair("only", "id,source,tag_string,file_url");

    let resp = crate::reqwest_client::HTTP.get(url).send().await?;

    match resp.error_for_status() {
        Ok(resp) => {
            let data: SafebooruResponse = resp.json().await?;

            if raw {
                ctx.say(data.file_url).await?;
            } else {
                ctx.send(
                    CreateReply::default().embed(
                        serenity::CreateEmbed::default()
                            .title(data.id.to_string())
                            .field("Tags", data.tag_string.replace('_', "\\_"), false)
                            .field("Source", data.source, false)
                            .url(format!("https://safebooru.donmai.us/posts/{}", data.id))
                            .image(data.file_url)
                            .color(0xfef9c3),
                    ),
                )
                .await?;
            }
        }

        Err(err) => {
            let err = color_eyre::eyre::Error::from(err);
            let valfisk_err = ValfiskError::new(&err, &ctx);
            valfisk_err.handle_log();
            valfisk_err.handle_report().await;

            let embed = serenity::CreateEmbed::default()
                .title("Could not fetch shiggy!")
                .description("An error occurred while fetching from the API.")
                .color(0xef4444)
                .footer(serenity::CreateEmbedFooter::new(valfisk_err.error_id));

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
