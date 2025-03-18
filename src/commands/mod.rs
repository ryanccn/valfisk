// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::Data;

mod fun;
mod moderation;
mod useful;
mod utils;

pub use utils::presence::restore as restore_presence;

macro_rules! command {
    ($category: ident, $name: ident) => {
        $category::$name::$name()
    };

    ($category: ident, $name: ident, $override: ident) => {
        $category::$name::$override()
    };
}

pub fn to_vec() -> Vec<poise::Command<Data, eyre::Report>> {
    vec![
        command!(useful, dig),
        command!(useful, lighthouse),
        command!(useful, remind),
        command!(useful, self_timeout),
        command!(useful, self_timeout, transparency),
        command!(useful, translate),
        command!(useful, translate, translate_private),
        command!(moderation, ban),
        command!(moderation, kick),
        command!(moderation, timeout),
        command!(fun, autoreply),
        command!(fun, owo),
        command!(fun, shiggy),
        command!(utils, ping),
        command!(utils, presence),
        command!(utils, rotate_color_roles),
        command!(utils, say),
        command!(utils, sysinfo),
        command!(utils, template_channel),
        command!(utils, version),
    ]
}
