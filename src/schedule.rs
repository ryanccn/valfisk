// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{EditRole, GuildId, GuildPagination, Http, RoleId};

use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::{task::JoinSet, time};

use chrono::{NaiveTime, TimeDelta, Timelike, Utc};
use eyre::{Result, eyre};

use crate::Data;

pub async fn rotate_color_role(
    http: &Http,
    guild: GuildId,
    role: RoleId,
) -> Result<HashSet<RoleId>> {
    if let Ok(mut role) = guild.role(http, role).await {
        let color: u32 = rand::random_range(0x000000..=0xffffff);

        role.edit(http, EditRole::default().colour(color)).await?;
        tracing::debug!(
            role = ?role.id,
            color = format!("{color:#x}"),
            "rotated role color"
        );

        let mut ret = HashSet::new();
        ret.insert(role.id);
        Ok(ret)
    } else {
        Ok(HashSet::new())
    }
}

pub async fn rotate_color_roles_guild(
    http: &Http,
    data: &Data,
    guild: GuildId,
) -> Result<HashSet<RoleId>> {
    if let Some(storage) = &data.storage {
        let guild_config = storage.get_config(guild).await?;
        let mut ret = HashSet::new();

        for role in &guild_config.random_color_roles {
            ret.extend(rotate_color_role(http, guild, *role).await?);
        }

        return Ok(ret);
    }

    Ok(HashSet::new())
}

pub async fn rotate_color_roles_global(http: &Http, data: &Data) -> Result<()> {
    let mut cursor: Option<GuildId> = None;

    loop {
        let guilds = http
            .get_guilds(cursor.map(GuildPagination::After), Some(50.try_into()?))
            .await?;

        if guilds.is_empty() {
            break;
        }

        cursor = guilds.last().map(|g| g.id);

        for guild in guilds {
            rotate_color_roles_guild(http, data, guild.id).await?;
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn run(http: Arc<Http>, data: Arc<Data>) -> Result<()> {
    let mut tasks: JoinSet<Result<()>> = JoinSet::new();

    tasks.spawn({
        let http = Arc::clone(&http);
        let data = Arc::clone(&data);

        async move {
            tracing::info_span!("rotate_color_roles")
                .in_scope(async || {
                    loop {
                        let now = Utc::now();

                        let next = (now + TimeDelta::days(1))
                            .with_time(NaiveTime::MIN)
                            .single()
                            .ok_or_else(|| eyre!("could not obtain next run time"))?;

                        tracing::trace!(?next, "next run");

                        time::sleep((next - now).to_std()?).await;

                        if let Err(err) = rotate_color_roles_global(&http, &data).await {
                            tracing::error!("{err:?}");
                        }

                        time::sleep(Duration::from_secs(1)).await;
                    }
                })
                .await
        }
    });

    tasks.spawn({
        let data = Arc::clone(&data);

        async move {
            tracing::info_span!("safe_browsing")
                .in_scope(async || {
                    loop {
                        let now = Utc::now();

                        let next = (now + TimeDelta::hours(1))
                            .with_minute(0)
                            .and_then(|t| t.with_second(0))
                            .and_then(|t| t.with_nanosecond(0))
                            .ok_or_else(|| eyre!("could not obtain next run time"))?;

                        tracing::trace!(?next, "next run");

                        time::sleep((next - now).to_std()?).await;

                        if let Some(safe_browsing) = &data.safe_browsing {
                            if let Err(err) = safe_browsing.update().await {
                                tracing::error!("{err:?}");
                            }
                        }

                        time::sleep(Duration::from_secs(1)).await;
                    }
                })
                .await
        }
    });

    tasks.join_all().await.into_iter().collect::<Result<()>>()?;

    Ok(())
}
