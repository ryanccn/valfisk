use owo_colors::OwoColorize;
use poise::FrameworkError;

use crate::{utils::error_handling::ValfiskError, Data};

pub async fn handle_error(err: &FrameworkError<'_, Data, color_eyre::eyre::Error>) {
    match err {
        FrameworkError::Setup { error, .. } => {
            eprintln!(
                "{} setting up client:\n{:#?}",
                "Encountered error".red(),
                error
            );
        }

        FrameworkError::EventHandler { error, .. } => {
            eprintln!(
                "{} handling event!\n{:#?}",
                "Encountered error".red(),
                error
            );
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
