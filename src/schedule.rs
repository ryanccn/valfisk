// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use jiff::{RoundMode, ToSpan, Unit, Zoned, ZonedRound, tz::TimeZone};
use poise::serenity_prelude::{EditRole, GuildId, GuildPagination, Http, RoleId};

use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::{task::JoinSet, time};

use eyre::Result;
use tracing::Instrument as _;

use crate::Data;

pub async fn rotate_color_role(
    http: &Http,
    guild: GuildId,
    role: RoleId,
) -> Result<Option<RoleId>> {
    let Ok(mut role) = guild.role(http, role).await else {
        return Ok(None);
    };

    let color: u32 = rand::random_range(0x000000..=0xffffff);

    role.edit(http, EditRole::default().colour(color)).await?;
    tracing::debug!(
        role = ?role.id,
        color = format!("{color:#x}"),
        "rotated role color"
    );

    Ok(Some(role.id))
}

pub async fn rotate_color_roles_guild(
    http: &Http,
    data: &Data,
    guild: GuildId,
) -> Result<HashSet<RoleId>> {
    let Some(storage) = &data.storage else {
        return Ok(HashSet::new());
    };

    let guild_config = storage.get_config(guild).await?;
    let mut ret = HashSet::new();

    for role in &guild_config.random_color_roles {
        ret.extend(rotate_color_role(http, guild, *role).await?);
    }

    Ok(ret)
}

pub async fn rotate_color_roles_global(http: &Http, data: &Data) -> Result<()> {
    let mut cursor: Option<GuildId> = None;

    loop {
        let guilds = http
            .get_guilds(
                cursor.map(GuildPagination::After),
                Some(50.try_into()?),
                None,
                None,
            )
            .await?;

        if guilds.is_empty() {
            break;
        }

        cursor = guilds.last().map(|g| g.id);

        for guild in guilds {
            if let Err(err) = rotate_color_roles_guild(http, data, guild.id).await {
                tracing::error!(guild = guild.id.get(), "{err:?}");
            }
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
            loop {
                let now = Zoned::now().with_time_zone(TimeZone::UTC);
                let next = (&now + 1.day()).start_of_day()?;
                tracing::trace!(?next, "next run at");

                time::sleep(Duration::try_from(next - now)?).await;

                if let Err(err) = rotate_color_roles_global(&http, &data).await {
                    tracing::error!("{err:?}");
                }

                time::sleep(Duration::from_secs(1)).await;
            }
        }
        .instrument(tracing::trace_span!("rotate_color_roles"))
    });

    tasks.spawn({
        let data = Arc::clone(&data);

        async move {
            loop {
                let now = Zoned::now().with_time_zone(TimeZone::UTC);
                let next = (&now + 1.hour()).round(
                    ZonedRound::new()
                        .smallest(Unit::Hour)
                        .mode(RoundMode::Trunc),
                )?;
                tracing::trace!(?next, "next run at");

                time::sleep(Duration::try_from(next - now)?).await;

                if let Some(safe_browsing) = &data.safe_browsing
                    && let Err(err) = safe_browsing.update().await
                {
                    tracing::error!("{err:?}");
                }

                time::sleep(Duration::from_secs(1)).await;
            }
        }
        .instrument(tracing::trace_span!("safe_browsing"))
    });

    while let Some(result) = tasks.join_next().await {
        () = result??;
    }

    Ok(())
}
