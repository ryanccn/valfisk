// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::serenity_prelude::{EditRole, Http, RoleId};

use rand::Rng as _;
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinSet, time};

use chrono::{NaiveTime, TimeDelta, Utc};
use eyre::{eyre, Result};

use crate::{config::CONFIG, Data};

pub async fn rotate_color_roles(
    http: &Arc<Http>,
    override_role: Option<RoleId>,
) -> Result<Vec<RoleId>> {
    let roles = match override_role {
        Some(role) => vec![role],
        None => CONFIG.random_color_roles.clone(),
    };

    if let Some(guild) = CONFIG.guild_id {
        for role in &roles {
            let mut role = guild.role(http, *role).await?;
            let color = {
                let mut rand = rand::thread_rng();
                rand.gen_range(0x000000..=0xffffff)
            };

            role.edit(http, EditRole::default().colour(color)).await?;
        }
    }

    Ok(roles)
}

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

                time::sleep((next - now).to_std()?).await;

                if let Err(err) = rotate_color_roles(&http, None).await {
                    tracing::error!("{err:?}");
                }

                time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    tasks.spawn({
        let data = data.clone();

        async move {
            let mut invtl = time::interval(Duration::from_secs(3600));

            loop {
                invtl.tick().await;

                if let Some(safe_browsing) = &data.safe_browsing {
                    if let Err(err) = safe_browsing.update().await {
                        tracing::error!("{err:?}");
                    }
                }
            }
        }
    });

    while let Some(res) = tasks.join_next().await {
        res??;
    }

    Ok(())
}
