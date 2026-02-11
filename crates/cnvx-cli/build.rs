//! Build script for the `cnvx` crate. It sets environment variables for the version and commit SHA.
//!
//! This allows for the version and the commit SHA to be baked into the binary at compile time.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=CNVX_VERSION");
    println!("cargo:rerun-if-env-changed=CNVX_COMMIT_SHA");

    if option_env!("CNVX_VERSION").is_none() {
        println!("cargo:rustc-env=CNVX_VERSION={}", env!("CARGO_PKG_VERSION"));
    }

    let profile = std::env::var("PROFILE").unwrap_or_default();

    if option_env!("CNVX_COMMIT_SHA").is_none()
        && profile == "debug"
        && let Some(sha) = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
    {
        println!("cargo:rustc-env=CNVX_COMMIT_SHA={sha}");
    }
}
