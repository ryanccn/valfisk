use log::error;

use poise::FrameworkError;

use crate::{utils::error_handling::ValfiskError, Data};

pub async fn handle_error(err: &FrameworkError<'_, Data, color_eyre::eyre::Report>) {
    match err {
        FrameworkError::Setup { error, .. } => {
            error!("{:?}", error);
        }

        FrameworkError::EventHandler { error, .. } => {
            error!("{:?}", error);
        }

        FrameworkError::Command { error, ctx, .. } => {
            ValfiskError::new(error, ctx).handle_all().await;
        }

        FrameworkError::CommandPanic { payload, ctx, .. } => {
            ValfiskError::new(payload, ctx).handle_all().await;
        }

        _ => {}
    }
}
