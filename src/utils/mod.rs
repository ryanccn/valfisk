pub mod actix;
pub mod error_handling;
pub mod serenity;

mod pluralize;
pub use pluralize::Pluralize;

use once_cell::sync::Lazy;
use poise::serenity_prelude::GuildId;

pub static GUILD_ID: Lazy<Option<GuildId>> = Lazy::new(|| {
    std::env::var("GUILD_ID")
        .ok()
        .and_then(|s| s.parse::<GuildId>().ok())
});
