// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::Data;

mod admin;
mod fun;
mod moderation;
mod useful;
mod utils;

pub mod restore {
    pub use super::admin::presence::restore as presence;
    pub use super::useful::remind::restore as reminders;
}

macro_rules! command {
    ($category: ident, $name: ident) => {
        $category::$name::$name()
    };

    ($category: ident, $name: ident, $override: ident) => {
        $category::$name::$override()
    };
}

pub fn all() -> Vec<poise::Command<Data, eyre::Report>> {
    vec![
        command!(useful, code_expand),
        command!(useful, dig),
        command!(useful, lighthouse),
        command!(useful, remind),
        command!(useful, self_timeout),
        command!(useful, translate),
        command!(useful, translate, translate_ephemeral),
        command!(moderation, ban),
        command!(moderation, kick),
        command!(moderation, timeout),
        command!(fun, autoreply),
        command!(fun, owo),
        command!(fun, shiggy),
        command!(utils, config),
        command!(utils, ping),
        command!(utils, rotate_color_roles),
        command!(utils, template_channel),
        command!(utils, version),
        command!(admin, presence),
        command!(admin, say),
        command!(admin, sysinfo),
    ]
}
