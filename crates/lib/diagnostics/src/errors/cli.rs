use miette::{
    Diagnostic,
    SourceSpan,
};
use owo_colors::OwoColorize;
use smol_str::SmolStr;
use thiserror::Error;

/// Prefix appended to all `CliError` messages.
pub const CLI_ERROR_PREFIX: &str = "CLI Error";

/// All possible errors that can occur as a result of the **command line
/// interface**. These errors are typically returned when the user provides
/// invalid input (e.g. an invalid target to be built like `bkg build
/// //:does_not_exist`).
#[derive(Debug, Error, Diagnostic, Clone)]
// #[derive(Debug, Error, Clone)]
pub enum CliError {
    /// Returned when the **target** specified by the user _does not exist_.
    /// To resolve this error, the user should check that the target is present
    /// within a `BUILD(.bazel)/BUCK` file in the current workspace.
    #[error(
        "{} {} {}{} {}",
        CLI_ERROR_PREFIX.blue(),
        "-".black(),
        "Target not found".red(),
        ":".black(),
        .target.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::cli::target_not_found),
        url(docsrs),
        help(
            "The target you have specified does not exist. Please check that the target is \
             present within a `BUILD/BUCK` in the current workspace."
        )
    )]
    TargetNotFound {
        /// The canonical command that was executed by the user (e.g. `bkg build
        /// //:does_not_exist`).
        #[source_code]
        command:      String,
        /// The target that was not found.
        target:       SmolStr,
        /// The closest matching targets that do exist (_did you mean?_) for
        /// the target that was not found.
        did_you_mean: Vec<String>,
        /// The span of the target within the command line input.
        #[label("Target not found")]
        span:         SourceSpan,
    },

    /// Returned when **multiple targets** specified by the user _do not exist_.
    /// To resolve this error, the user should check that the targets are
    /// present within a `BUILD(.bazel)/BUCK` file in the current workspace.
    #[error(
        "{} {} {}{} {}",
        CLI_ERROR_PREFIX.blue(),
        "-".black(),
        "Targets not found".red(),
        ":\n".black(),
        .targets.iter().map(|target| format!("  - {}", target.yellow().italic())).collect::<Vec<_>>().join("\n")
    )]
    #[diagnostic(
        code(byakugan::cli::targets_not_found),
        url(docsrs),
        help(
            "The targets you have specified do not exist. Please check that the targets are \
             present within a `BUILD/BUCK` in the current workspace."
        )
    )]
    TargetsNotFound {
        /// The canonical command that was executed by the user (e.g. `bkg build
        /// //:does_not_exist //:also_does_not_exist`).
        #[source_code]
        command:      String,
        /// The targets that were not found.
        targets:      Vec<String>,
        /// The set of closest matching targets that do exist (_did you mean?_)
        /// for each target that was not found.
        did_you_mean: Vec<String>,
        /// The span of the targets within the command line input.
        #[label("Targets not found")]
        span:         SourceSpan,
    },

    /// Returned when **multiple targets** are specified by the user but the
    /// command only supports a single target. To resolve this error, the user
    /// should check that they are only specifying a single target.
    ///
    /// For example, the `bkg run` command only supports a single target, so
    /// specifying multiple targets like `bkg run //:foo //:bar` will result in
    /// this error.
    #[error(
        "{} {} {}",
        CLI_ERROR_PREFIX.blue(),
        "-".black(),
        "Multiple targets specified but only a single target is supported. Please check that you are \
          only specifying a single target."
    )]
    #[diagnostic(
        code(byakugan::cli::multiple_targets_not_supported),
        url(docsrs),
        help(
            "Multiple targets specified but only a single target is supported. Please check that \
             you are only specifying a single target."
        )
    )]
    MultipleTargetsNotSupported {
        /// The canonical command that was executed by the user (e.g. `bkg run
        /// //:foo //:bar`).
        #[source_code]
        command: String,
        /// The span of the targets within the command line input.
        #[label("Multiple targets not supported")]
        span:    SourceSpan,
    },

    /// Returned when no **build system** can be detected from the current
    /// directory or any of its parent directories checked recursively up to
    /// the root directory. To resolve this error, the user should check
    /// that they are executing the command from within a valid workspace (e.g.
    /// a directory containing a `Cargo.toml` file for the `cargo` build system,
    /// or analogous configuration files for other build systems).
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
            "No build system detected. Please check that you are executing the command from \
             within a valid workspace (e.g. a directory containing a `Cargo.toml` file for the \
             `cargo` build system, or analogous configuration files for other build systems)."
        )
    )]
    NoBuildSystemDetected {
        /// The canonical command that was executed by the user (e.g. `bkg build
        /// //:does_not_exist`).
        #[source_code]
        command: String,
    },
}

// impl Diagnostic for CliError {
//     fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
//         match self {
//             Self::TargetNotFound { .. } =>
// Some(Box::new("byakugan::cli::target_not_found")),
// Self::TargetsNotFound { .. } =>
// Some(Box::new("byakugan::cli::targets_not_found")),
// Self::MultipleTargetsNotSupported { .. } => {
// Some(Box::new("byakugan::cli::multiple_targets_not_supported"))             }
//             Self::NoBuildSystemDetected { .. } => {
//                 Some(Box::new("byakugan::cli::no_build_system_detected"))
//             }
//         }
//     }

//     fn severity(&self) -> Option<miette::Severity> {
//         Some(miette::Severity::Error)
//     }

//     fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
//         match self {
//             Self::TargetNotFound { .. } => Some(Box::new(
//                 "The target you have specified does not exist. Please check
// that the target is \                  present within a `BUILD/BUCK` in the
// current workspace.",             )),
//             Self::TargetsNotFound { .. } => Some(Box::new(
//                 "The targets you have specified do not exist. Please check
// that the targets are \                  present within a `BUILD/BUCK` in the
// current workspace.",             )),
//             Self::MultipleTargetsNotSupported { .. } => Some(Box::new(
//                 "Multiple targets specified but only a single target is
// supported. Please check \                  that you are only specifying a
// single target.",             )),
//             Self::NoBuildSystemDetected { .. } => Some(Box::new(
//                 "No build system detected. Please check that you are
// executing the command from \                  within a valid workspace (e.g.
// a directory containing a `Cargo.toml` file for \                  the `cargo`
// build system, or analogous configuration files for other build \
// systems).",             )),
//         }
//     }

//     fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
//         match self {
//             Self::TargetNotFound { .. } => Some(Box::new("https://docs.rs/byakugan")),
//             Self::TargetsNotFound { .. } => Some(Box::new("https://docs.rs/byakugan")),
//             Self::MultipleTargetsNotSupported { .. } => Some(Box::new("https://docs.rs/byakugan")),
//             Self::NoBuildSystemDetected { .. } => Some(Box::new("https://docs.rs/byakugan")),
//         }
//     }

//     fn source_code(&self) -> Option<&dyn miette::SourceCode> {
//         match self {
//             Self::TargetNotFound { command, .. } => Some(command),
//             Self::TargetsNotFound { command, .. } => Some(command),
//             Self::MultipleTargetsNotSupported { command, .. } =>
// Some(command),             Self::NoBuildSystemDetected { command, .. } =>
// Some(command),         }
//     }

//     fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> +
// '_>> {         match self {
//             Self::TargetNotFound { command, span, .. } => {
//                 Some(Box::new(std::iter::once(miette::LabeledSpan::new(
//                     Some(command.clone()),
//                     span.offset(),
//                     span.offset() + span.len(),
//                 ))))
//             }
//             Self::TargetsNotFound { command, span, .. } => {
//                 Some(Box::new(std::iter::once(miette::LabeledSpan::new(
//                     Some(command.clone()),
//                     span.offset(),
//                     span.offset() + span.len(),
//                 ))))
//             }
//             Self::MultipleTargetsNotSupported { command, span, .. } => {
//                 Some(Box::new(std::iter::once(miette::LabeledSpan::new(
//                     Some(command.clone()),
//                     span.offset(),
//                     span.offset() + span.len(),
//                 ))))
//             }
//             Self::NoBuildSystemDetected { .. } => None,
//         }
//     }

//     fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn
// Diagnostic> + 'a>> {         None
//     }

//     fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
//         match self {
//             Self::TargetNotFound { .. } => Some(self),
//             Self::TargetsNotFound { .. } => Some(self),
//             Self::MultipleTargetsNotSupported { .. } => Some(self),
//             Self::NoBuildSystemDetected { .. } => Some(self),
//         }
//     }
// }
