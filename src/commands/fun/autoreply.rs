// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::{Result, eyre};
use poise::{
    CreateReply,
    serenity_prelude::{CreateEmbed, Timestamp},
};

use crate::Context;

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    slash_command,
    guild_only,
    subcommands("list", "add", "delete", "delete_all"),
    subcommand_required,
    default_member_permissions = "MANAGE_GUILD"
)]
#[allow(clippy::unused_async)]
pub async fn autoreply(ctx: Context<'_>) -> Result<()> {
    Ok(())
}

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_GUILD")]
async fn list(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    let data = ctx
        .data()
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?
        .getall_autoreply()
        .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Autoreply")
                .description(
                    data.into_iter()
                        .map(|(k, v)| format!("`{k}` → `{v}`"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_GUILD")]
async fn add(
    ctx: Context<'_>,
    #[description = "The keyword included in the message (regex)"] keyword: String,
    #[description = "The response to reply to the message"] reply: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    ctx.data()
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?
        .add_autoreply(&keyword, &reply)
        .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Added autoreply keyword")
                .description(format!("`{keyword}` → `{reply}`"))
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_GUILD")]
async fn delete(
    ctx: Context<'_>,
    #[description = "The keyword included in the message (regex)"] keyword: String,
) -> Result<()> {
    ctx.defer_ephemeral().await?;

    ctx.data()
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?
        .del_autoreply(&keyword)
        .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Deleted autoreply keyword")
                .description(format!("`{keyword}`"))
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}

#[tracing::instrument(skip(ctx), fields(channel = ctx.channel_id().get(), author = ctx.author().id.get()))]
#[poise::command(
    rename = "delete-all",
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_GUILD"
)]
async fn delete_all(ctx: Context<'_>) -> Result<()> {
    ctx.defer_ephemeral().await?;

    ctx.data()
        .storage
        .as_ref()
        .ok_or_else(|| eyre!("storage is not available"))?
        .delall_autoreply()
        .await?;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Deleted all autoreply keywords")
                .timestamp(Timestamp::now())
                .color(0x63e6be),
        ),
    )
    .await?;

    Ok(())
}
