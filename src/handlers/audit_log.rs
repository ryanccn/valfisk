// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::Result;
use poise::serenity_prelude as serenity;

use crate::commands::moderation::action;

fn container(
    entry: &serenity::AuditLogEntry,
    target: serenity::UserId,
) -> Option<serenity::CreateContainer<'static>> {
    use serenity::audit_log::{Action, Change, MemberAction};

    match &entry.action {
        Action::Member(MemberAction::BanAdd) => Some(action::container("Ban", target, 0xda77f2)),

        Action::Member(MemberAction::BanRemove) => {
            Some(action::container("Unban", target, 0xda77f2))
        }

        Action::Member(MemberAction::Kick) => Some(action::container("Kick", target, 0xf783ac)),

        Action::Member(MemberAction::Update) => {
            let (old, new) = entry.changes.iter().find_map(|c| match *c {
                Change::CommunicationDisabledUntil { old, new } => Some((old, new)),
                _ => None,
            })?;

            match (old, new) {
                (_, Some(until)) => Some(action::field(
                    action::container("Timeout", target, 0x9775fa),
                    "Until",
                    serenity::FormattedTimestamp::new(until, None),
                )),
                (Some(_), None) => Some(action::container("Timeout removed", target, 0x9775fa)),
                (None, None) => None,
            }
        }

        &_ => None,
    }
}

#[tracing::instrument(skip_all, fields(id = entry.id.get(), guild_id = guild_id.get()))]
pub async fn handle(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
) -> Result<()> {
    if let Some(moderator) = entry.user_id
        && moderator != ctx.cache.current_user().id
        && let Some(target) = entry.target_id.map(|id| serenity::UserId::new(id.get()))
        && let Some(mut container) = container(entry, target)
        && let Some(storage) = &ctx.data::<crate::Data>().storage
    {
        if let Some(reason) = &entry.reason {
            container = action::field(container, "Reason", reason);
        }

        let guild_config = storage.get_config(guild_id).await?;
        action::log(
            &ctx.http,
            &guild_config,
            container,
            moderator,
            Some("Audit log"),
        )
        .await?;
    }

    Ok(())
}
