use std::{env, time::SystemTime};

fn main() {
    println!(
        "cargo:rustc-env=METADATA_HOST={}",
        env::var("HOST").unwrap()
    );
    println!(
        "cargo:rustc-env=METADATA_TARGET={}",
        env::var("TARGET").unwrap()
    );

    println!("cargo:rerun-if-changed-env=TARGET");
    println!("cargo:rerun-if-changed-env=HOST");

    let last_modified = env::var("METADATA_LAST_MODIFIED").unwrap_or_else(|_| {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    });

    println!("cargo:rustc-env=METADATA_LAST_MODIFIED={last_modified}");
    println!("cargo:rerun-if-changed-env=METADATA_LAST_MODIFIED");
}
