//! Build script to embed commit hash and build date at compile time.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get commit from env var (CI) or git command (local dev)
    let commit = env::var("BUILD_COMMIT").unwrap_or_else(|_| {
        Command::new("git")
            .args(["rev-parse", "--short=7", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });

    // Get date from env var (CI) or current date (local dev)
    let date = env::var("BUILD_DATE").unwrap_or_else(|_| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    // Write to OUT_DIR for inclusion
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    fs::write(
        &dest_path,
        format!(
            r#"pub const BUILD_COMMIT: &str = "{}";
pub const BUILD_DATE: &str = "{}";"#,
            commit, date
        ),
    )
    .unwrap();

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=BUILD_COMMIT");
    println!("cargo:rerun-if-env-changed=BUILD_DATE");
}
