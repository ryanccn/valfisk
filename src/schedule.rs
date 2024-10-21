use poise::serenity_prelude::{EditRole, Http, RoleId};

use once_cell::sync::Lazy;
use rand::Rng as _;
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinSet, time::sleep};

use chrono::{NaiveTime, TimeDelta, Utc};
use eyre::{eyre, Result};

use crate::utils;

static RANDOM_COLOR_ROLES: Lazy<Vec<RoleId>> = Lazy::new(|| {
    std::env::var("RANDOM_COLOR_ROLES")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|f| f.trim().parse::<RoleId>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
});

pub async fn rotate_color_roles(
    http: &Arc<Http>,
    override_role: Option<RoleId>,
) -> Result<Vec<RoleId>> {
    let roles = match override_role {
        Some(role) => vec![role],
        None => RANDOM_COLOR_ROLES.clone(),
    };

    if let Some(guild) = *utils::GUILD_ID {
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

pub async fn start(http: Arc<Http>) -> Result<()> {
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

                sleep((next - now).to_std()?).await;
                rotate_color_roles(&http, None).await?;
                sleep(Duration::from_secs(1)).await;
            }
        }
    });

    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(res) => match res {
                Ok(()) => {}
                Err(err) => {
                    return Err(err);
                }
            },
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
