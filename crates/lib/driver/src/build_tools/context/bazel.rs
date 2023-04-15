use std::process::Command;

/// Check if `bazel` or `bazelisk` is installed and available on the `PATH`
pub fn is_installed() -> bool {
    Command::new("bazel").arg("--version").output().is_ok() ||
        Command::new("bazelisk").arg("--version").output().is_ok()
}
