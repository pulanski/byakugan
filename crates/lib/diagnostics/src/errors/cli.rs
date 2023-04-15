use miette::{Diagnostic, SourceSpan};
use owo_colors::OwoColorize;
use smol_str::SmolStr;
use thiserror::Error;

/// Prefix appended to all `CliError` messages.
pub const CLI_ERROR_PREFIX: &str = "CLI Error";

/// All possible errors that can occur as a result of the **command line interface**.
/// These errors are typically returned when the user provides invalid input (e.g. an
/// invalid target to be built like `bkg build //:does_not_exist`).
#[derive(Debug, Error, Diagnostic, Clone)]
pub enum CliError {
    /// Returned when the **target** specified by the user _does not exist_.
    /// To resolve this error, the user should check that the target is present
    /// within a `BUILD(.bazel)/BUCK` file in the current workspace.
    #[error(
        "{} {} {}{} {}",
        CLI_ERROR_PREFIX.blue(),
        "-".black(),
        "Target not found: ".red(),
        ":".black(),
        .target.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::cli::target_not_found),
        url(docsrs),
        help(
            "The target you have specified does not exist. Please check that the target is present within \
              a `BUILD/BUCK` in the current workspace."
        )
    )]
    TargetNotFound {
        /// The canonical command that was executed by the user (e.g. `bkg build //:does_not_exist`).
        #[source_code]
        command: String,
        /// The target that was not found.
        target: SmolStr,
        /// The span of the target within the command line input.
        #[label("Target not found")]
        span: SourceSpan,
    },

    /// Returned when no **build system** can be detected from the current directory or any of its
    /// parent directories checked recursively up to the root directory. To resolve this error, the
    /// user should check that they are executing the command from within a valid workspace (e.g.
    /// a directory containing a `Cargo.toml` file for the `cargo` build system, or analogous
    /// configuration files for other build systems).
    #[error(
        "{} {} {}",
        CLI_ERROR_PREFIX.blue(),
        "-".black(),
        "No build system detected. Please check that you are executing the command from within a valid \
          workspace (e.g. a directory containing a `Cargo.toml` file for the `cargo` build system, or \
          analogous configuration files for other build systems)."
    )]
    #[diagnostic(
        code(byakugan::cli::no_build_system_detected),
        url(docsrs),
        help(
            "No build system detected. Please check that you are executing the command from within a valid \
              workspace (e.g. a directory containing a `Cargo.toml` file for the `cargo` build system, or \
              analogous configuration files for other build systems)."
        )
    )]
    NoBuildSystemDetected {
        /// The canonical command that was executed by the user (e.g. `bkg build //:does_not_exist`).
        #[source_code]
        command: String,
    },
}
