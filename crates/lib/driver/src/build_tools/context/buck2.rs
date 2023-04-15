use miette::Result;

pub mod query {
    use miette::IntoDiagnostic;

    use super::*;

    /// Use `buck2 query` to collect all build targets in the **current workspace/cell**.
    ///
    /// # Returns
    ///
    /// A list of all build targets in the current workspace/cell.
    #[tracing::instrument(name = "buck2 query //...")]
    pub fn all_targets() -> Result<Vec<String>> {
        // pub async fn all_targets() -> Result<Vec<String>> { // TODO: in the future, make async
        // TODO: in the future, convert this to a Result<Vec<Target>> where Target is a struct
        // that contains target information (e.g. label, rule, etc.)
        tracing::debug!("Querying all build targets in the current workspace/cell...");

        let output = std::process::Command::new("buck2")
            .arg("query")
            .arg("...")
            .output()
            .into_diagnostic()?;

        let stdout = String::from_utf8(output.stdout).into_diagnostic()?;

        let targets = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>();

        tracing::debug!(
            "Found {} build targets in the current workspace/cell",
            targets.len()
        );

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
