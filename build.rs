use std::{env, time::SystemTime};

fn main() {
    println!(
        "cargo:rustc-env=METADATA_TARGET={}",
        env::var("TARGET").unwrap()
    );

    let last_modified = env::var("METADATA_LAST_MODIFIED").unwrap_or(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    );

    println!("cargo:rustc-env=METADATA_LAST_MODIFIED={last_modified}");
}
