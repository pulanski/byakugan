pub use errors::*;

/// Prefix appended to all `RudolphDiagnostic` diagnostics.
pub static RUDOLPH_DIAGNOSTIC_PREFIX: &str = "Rudolph";

mod utils {
    use rand::Rng;

    /// Black ANSI escape code.
    pub static BLACK: &str = "\u{001B}[38;5;0m";

    // Define ANSI escape codes for different colors
    static RESET_COLOR: &str = "\u{001B}[0m"; // Reset color to default

    pub fn syntax_highlight_path(path: &str) -> String {
        // Generate random color codes for arbitrary depth of directories
        let mut rng = rand::thread_rng();

        let depth = path.split('/').count();

        let mut colors = Vec::new();
        for _ in 0..depth {
            let color = format!("\u{001B}[38;5;{}m", rng.gen_range(0..=255));
            colors.push(color);
        }

        // Split path into components and colorize each component
        let mut colored_path = String::new();
        for (i, component) in path.split('/').enumerate() {
            colored_path.push_str(&colors[i]);
            colored_path.push_str(component);
            colored_path.push_str(BLACK);
            colored_path.push('/');
            colored_path.push_str(RESET_COLOR);
        }

        colored_path

        // path.split('/')
        //     .map(|component| format!("{}{}", component.green().bold(),
        // "/".black()))     .collect::<String>()
    }
}

mod errors {
    use owo_colors::OwoColorize;
    use thiserror::Error;

    use super::utils::syntax_highlight_path;

    // use super::RUDOLPH_DIAGNOSTIC_PREFIX;

    /// Prefix appended to all `RudolphError` errors.
    pub const RUDOLPH_ERROR_PREFIX: &str = " ERROR ";

    pub fn error_message(message: &str, from: Option<&str>) -> String {
        format!(
            "{} {} {}",
            // "{} {} {} {}",
            // RUDOLPH_DIAGNOSTIC_PREFIX.red().bold(),
            RUDOLPH_ERROR_PREFIX.black().on_red(),
            "-".black(),
            message
        )
    }

    fn not_yet_implemented_error_message(feature: &str) -> String {
        format!(
            "{} {} {} is in development{}",
            error_message("Not yet implemented".blue().to_string().as_str(), None),
            "-".black(),
            feature.green().bold(),
            ".".black()
        )
    }

    fn third_party_dir_does_not_exist_error_message(path: &str) -> String {
        format!(
            "{} {} {} {}\n\n  {}",
            error_message("Third-party directory does not exist".red().to_string().as_str(), None),
            "-".black(),
            syntax_highlight_path(path),
            " was not found.",
            "Please ensure that the third-party directory exists.".black()
        )
    }

    fn failed_to_build_workspace_error_message() -> String {
        format!(
            "{} {} {} {}",
            error_message("Failed to build workspace".red().to_string().as_str(), None),
            "-".black(),
            "Failed to build workspace with newly introduced".black(),
            " third-party dependencies.".red(),
        )
    }

    /// An error that occurs when Rudolph is unable to perform a task.
    /// This error is used to indicate that Rudolph was unable to perform a task
    /// due to an internal error.
    #[derive(Error, Debug)]
    pub enum RudolphError {
        // #[error("failed to read file: {0}")]
        // ReadFileError(String),
        // #[error("failed to write file: {0}")]
        // WriteFileError(String),
        // #[error("failed to create directory: {0}")]
        // CreateDirError(String),
        // #[error("failed to remove directory: {0}")]
        // RemoveDirError(String),
        // #[error("failed to remove file: {0}")]
        // RemoveFileError(String),
        // #[error("failed to rename file: {0}")]
        // RenameFileError(String),
        // #[error("failed to create process: {0}")]
        // CreateProcessError(String),
        // #[error("failed to wait on process: {0}")]
        // WaitOnProcessError(String),
        // #[error("failed to kill process: {0}")]
        // KillProcessError(String),
        // #[error("failed to read process output: {0}")]
        // ReadProcessOutputError(String),
        // #[error("failed to read process status: {0}")]
        // ReadProcessStatusError(String),
        // #[error("failed to read process id: {0}")]
        // ReadProcessIdError(String),
        // #[error("failed to read process exit code: {0}")]
        // ReadProcessExitCodeError(String),
        // #[error("failed to read process stdout: {0}")]
        // ReadProcessStdoutError(String),
        // #[error("failed to read process stderr: {0}")]
        // ReadProcessStderrError(String),
        // #[error("failed to read process stdin: {0}")]
        // ReadProcessStdinError(String),
        // #[error("failed to write process stdin: {0}")]
        // WriteProcessStdinError(String),
        #[error("{}", third_party_dir_does_not_exist_error_message(_0))]
        ThirdPartyDirDoesNotExist(String),

        #[error("{}", failed_to_build_workspace_error_message())]
        FailedToBuildWorkspace,
        #[error("{}", not_yet_implemented_error_message(_0))]
        NotYetImplemented(String),
    }
}
