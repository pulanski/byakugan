use miette::{
    Diagnostic,
    SourceSpan,
};
use owo_colors::OwoColorize;
use smol_str::SmolStr;
use thiserror::Error;

/// Prefix appended to all `SyntaxError` messages.
pub const SYNTAX_ERROR_PREFIX: &str = "Syntax Error";

/// All possible **syntax errors** that can occur within the user's input.
/// These errors are typically returned when the user provides invalid input
/// (e.g. an invalid target to be built like `bkg build
/// //\invalid_\\target_\\syntax`).
#[derive(Debug, Error, Diagnostic, Clone)]
pub enum SyntaxError {
    /// Returned when the **repo name** specified by the user within a **label**
    /// for a target is _invalid_. To resolve this error, the user should
    /// check that the repo name is valid and does not contain any invalid
    /// characters and adheres to the following format: `@<repo_name>` (e.g.
    /// `@my_repo`) where `<repo_name>` ∈ `@[\\w\\-.][\\w\\-.~]*`.
    #[error(
        "{} {} {}{} {}",
        SYNTAX_ERROR_PREFIX.blue(),
        "-".black(),
        "Invalid repo name: ".red(),
        ":".black(),
        .repo.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::syntax::invalid_repo_name),
        url(docsrs),
        help(
            "The repo name you have specified is syntactically invalid. Please check that the \
             repo name is valid and does not contain any invalid characters, adhering to the \
             following format: `@<repo_name>` (e.g. `@my_repo`) where `<repo_name>` ∈ \
             @[\\w\\-.][\\w\\-.~]*"
        )
    )]
    InvalidRepoName {
        /// The **label** (e.g. `@my_repo//:target`) that was being parsed.
        #[source_code]
        label: String,
        /// The repo name that was not found (e.g. `@invalid∈_repo∈_name∈`).
        repo:  SmolStr,
        /// The span of the target within the command line input.
        #[label("Invalid repo name")]
        span:  SourceSpan,
    },

    /// Returned when the **target name** specified by the user within a
    /// **label** for a target is _invalid_. To resolve this error, the user
    /// should check that the target name is valid and does not contain any
    /// invalid characters and adheres to the following format:
    /// `<target_name>` (e.g. `:target`) where `<target_name>` ∈ `:[\\w\\-.
    /// ][\\w\\-.~]*`.
    #[error(
        "{} {} {}{} {}",
        SYNTAX_ERROR_PREFIX.blue(),
        "-".black(),
        "Invalid target name: ".red(),
        ":".black(),
        .target.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::syntax::invalid_target_name),
        url(docsrs),
        help(
            "The target name you have specified is syntactically invalid. Please check that the \
             target name is valid and does not contain any invalid characters, adhering to the \
             following format: `<target_name>` (e.g. `:target`) where `<target_name>` ∈ \
             `:[\\w\\-.][\\w\\-.~]*`"
        )
    )]
    InvalidTargetName {
        /// The **label** (e.g. `@my_repo//:target`) that was being parsed.
        #[source_code]
        label:  String,
        /// The target name that was not found (e.g. `:invalid∈_target∈_name∈`).
        target: SmolStr,
        /// The span of the target within the command line input.
        #[label("Invalid target name")]
        span:   SourceSpan,
    },

    /// Returned when the **label** specified by the user for a target is
    /// _invalid_. To resolve this error, the user should check that the
    /// label is valid and does not contain any syntax errors. A valid label
    /// adheres to the following format: `<repo_name>`? `:<target_name>`
    /// (e.g. `@my_repo//:target`) where `<repo_name>` ∈ `@[\\w\\-.][\\w\\-.
    /// ~]*` and `<target_name>` ∈ `:[\\w\\-.][\\w\\-.~]*`.
    #[error(
        "{} {} {}{} {}",
        SYNTAX_ERROR_PREFIX.blue(),
        "-".black(),
        "Invalid label: ".red(),
        ":".black(),
        .label.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::syntax::invalid_label),
        url(docsrs),
        help(
            "The label you have specified is syntactically invalid. Please check that the label \
             is valid and does not contain any syntax errors, adhering to the following format: \
             `<repo_name>`? `:<target_name>` (e.g. `@my_repo//:target`) where `<repo_name>` ∈ \
             `@[\\w\\-.][\\w\\-.~]*` and `<target_name>` ∈ `:[\\w\\-.][\\w\\-.~]*`"
        )
    )]
    InvalidLabel {
        /// The **label** (e.g. `@my_repo//:target`) that was being parsed.
        #[source_code]
        label: String,
        /// The span of the label within the command line input.
        #[label("Invalid label")]
        span:  SourceSpan,
    },
}
