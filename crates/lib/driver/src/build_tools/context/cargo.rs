use std::process::Command;

/// Check if `cargo` is installed and available on the `PATH`
pub fn is_installed() -> bool {
    Command::new("cargo").arg("--version").output().is_ok()
}
