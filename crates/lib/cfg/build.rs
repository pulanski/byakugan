#![allow(non_snake_case)]

// use chrono::Utc;
use std::env::consts::{ARCH, FAMILY, OS};
use std::{error::Error, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    COMMIT_HASH()?;
    BUILD_DATE();
    TARGET_TRIPLE();

    Ok(())
}

fn COMMIT_HASH() -> Result<(), Box<dyn Error>> {
    println!(
        "cargo:rustc-env=LEAFC_COMMIT_HASH={}",
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()?
                .stdout,
        )?
    );

    Ok(())
}

fn TARGET_TRIPLE() {
    println!("cargo:rustc-env=LEAFC_TARGET_TRIPLE={OS}-{FAMILY}-{ARCH}");
}

fn BUILD_DATE() {
    println!(
        "cargo:rustc-env=LEAFC_BUILD_DATE={}",
        "2021-08-01" // Utc::now().format("%Y-%m-%d")
    );
}
