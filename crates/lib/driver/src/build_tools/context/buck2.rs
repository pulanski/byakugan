use miette::Result;

use super::is_binary_installed;

pub mod query {
    use miette::IntoDiagnostic;

    use super::*;

    /// Use `buck2 query` to collect all build targets in the **current
    /// workspace/cell**.
    ///
    /// # Returns
    ///
    /// A list of all build targets in the current workspace/cell.
    pub fn all_targets() -> Result<Vec<String>> {
        // pub fn all_targets() -> Result<Vec<Target>> { // TODO:
        tracing::debug!("Querying all build targets in the current workspace/cell...");

        let output = std::process::Command::new("buck2")
            .arg("query")
            .arg("...")
            .output()
            .into_diagnostic()?;

        let stdout = String::from_utf8(output.stdout).into_diagnostic()?;

        let targets = stdout.lines().map(|line| line.trim().to_string()).collect::<Vec<_>>();

        tracing::debug!("Found {} build targets in the current workspace/cell", targets.len());

        if targets.is_empty() {
            tracing::warn!("No build targets found in the current workspace/cell");
        }

        if tracing::level_enabled!(tracing::Level::TRACE) {
            tracing::trace!("Found build targets:");
            for target in &targets {
                tracing::trace!("  {}", target);
            }
        }

        // let targets: Vec<Target> = targets::from_buck2_query_output(stdout);

        Ok(targets)
    }
}

/// Check if `buck` or `buck2` is installed and available on the `PATH`.
///
/// **NOTE**: This operation is cached between runs of the program, meaning that
/// the first time this function is called, it will be a blocking operation
/// performed at runtime, however from that point on, it will be a fast lookup
/// operation reading from a cached value.
pub fn is_installed() -> bool {
    is_binary_installed("buck") || is_binary_installed("buck2")
}
