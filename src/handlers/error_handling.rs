// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use poise::FrameworkError;

use crate::{Data, utils::error_handling::ValfiskError};

#[tracing::instrument(skip(err))]
pub async fn handle_error(err: FrameworkError<'_, Data, eyre::Report>) {
    match err {
        FrameworkError::Command { error, ctx, .. } => {
            ValfiskError::error(&error, &ctx).handle_all().await;
        }

        FrameworkError::CommandPanic { payload, ctx, .. } => {
            ValfiskError::panic(payload.as_ref(), &ctx)
                .handle_all()
                .await;
        }

        _ => {
            tracing::error!("{err:?}");
        }
    }
}
