// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::env;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

include!("src/ucd/build.rs");

fn main() -> Result<()> {
    println!("cargo:rustc-env=METADATA_TARGET={}", env::var("TARGET")?);
    println!("cargo:rustc-env=METADATA_HOST={}", env::var("HOST")?);

    println!("cargo::rerun-if-env-changed=VALFISK_GENERATE_UCD");
    if env::var("VALFISK_GENERATE_UCD").is_ok_and(|s| s == "1") {
        ucd::generate()?;
    }

    Ok(())
}
