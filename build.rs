// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env;

fn main() {
    println!(
        "cargo:rustc-env=METADATA_TARGET={}",
        env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rustc-env=METADATA_HOST={}",
        env::var("HOST").unwrap()
    );
}
