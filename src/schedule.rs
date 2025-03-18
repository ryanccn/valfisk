// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{EditRole, Http, RoleId};

use rand::Rng as _;
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinSet, time};
use tracing::Instrument as _;

use chrono::{NaiveTime, TimeDelta, Timelike, Utc};
use eyre::{Result, eyre};

use crate::{Data, config::CONFIG};

#[tracing::instrument(skip(http))]
pub async fn rotate_color_roles(
    http: &Arc<Http>,
    override_role: Option<RoleId>,
) -> Result<Vec<RoleId>> {
    let roles = override_role.map_or_else(|| CONFIG.random_color_roles.clone(), |role| vec![role]);

    if let Some(guild) = CONFIG.guild_id {
        for role in &roles {
            let mut role = guild.role(http, *role).await?;
            let color: u32 = {
                let mut rand = rand::rng();
                rand.random_range(0x000000..=0xffffff)
            };

            role.edit(http, EditRole::default().colour(color)).await?;
            tracing::info!("Rotated role {} color => {:#x}", role.id, color);
        }
    }

    Ok(roles)
}

#[tracing::instrument(skip_all)]
pub async fn start(http: Arc<Http>, data: Arc<Data>) -> Result<()> {
    let mut tasks: JoinSet<Result<()>> = JoinSet::new();

    tasks.spawn({
        let http = http.clone();

        async move {
            loop {
                let now = Utc::now();

                let next = (now + TimeDelta::days(1))
                    .with_time(NaiveTime::MIN)
                    .single()
                    .ok_or_else(|| eyre!("could not obtain next run time"))?;

                tracing::trace!("Next run at {next}");

                time::sleep((next - now).to_std()?).await;

                if let Err(err) = rotate_color_roles(&http, None).await {
                    tracing::error!("{err:?}");
                }

                time::sleep(Duration::from_secs(1)).await;
            }
        }
        .instrument(tracing::info_span!("rotate_color_roles"))
    });

    tasks.spawn({
        let data = data.clone();

        async move {
            loop {
                let now = Utc::now();

                let next = (now + TimeDelta::hours(1))
                    .with_minute(0)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .ok_or_else(|| eyre!("could not obtain next run time"))?;

                tracing::trace!("Next run at {next}");

                time::sleep((next - now).to_std()?).await;

                if let Some(safe_browsing) = &data.safe_browsing {
                    if let Err(err) = safe_browsing.update().await {
                        tracing::error!("{err:?}");
                    }
                }

                time::sleep(Duration::from_secs(1)).await;
            }
        }
        .instrument(tracing::info_span!("safe_browsing"))
    });

    while let Some(res) = tasks.join_next().await {
        res??;
    }

    Ok(())
}
