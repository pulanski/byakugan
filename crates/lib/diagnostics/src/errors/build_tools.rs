use miette::Diagnostic;
use owo_colors::OwoColorize;
use thiserror::Error;

/// Prefix appended to all `ToolchainError` diagnostics.
pub const TOOLCHAIN_ERROR_PREFIX: &str = "Toolchain Error";

/// An error that occurs when a toolchain is not found (e.g. `buck2`, `bazel`,
/// `cargo`, etc.). This error is used to indicate that a toolchain is not
/// installed on the system or is not found in the `PATH` and therefore cannot
/// be used to execute the given task. To fix this error, the user must ensure
/// that the toolchain is installed and available in the `PATH`.
#[derive(Debug, Error, Diagnostic, Clone)]
pub enum ToolchainError {
    /// Error returned when neither `buck2` nor `buck` is found in the `PATH`.
    #[error(
        "{} {} {}",
        TOOLCHAIN_ERROR_PREFIX.blue(),
        "-".black(),
        "buck2 not found".red(),
    )]
    #[diagnostic(
        code(byakugan::toolchain::buck2_not_found),
        url(docsrs),
        help(
            "Neither `buck2` nor `buck` was found. Please ensure that either `buck2` or `buck` is \
             installed and available in the `PATH`."
        )
    )]
    Buck2NotFound,

    /// Error returned when neither `bazel` nor `bazelisk` is found in the
    /// `PATH`.
    #[error(
        "{} {} {}",
        TOOLCHAIN_ERROR_PREFIX.blue(),
        "-".black(),
        "bazel not found".red(),
    )]
    #[diagnostic(
        code(byakugan::toolchain::bazel_not_found),
        url(docsrs),
        help(
            "Neither `bazel` nor `bazelisk` was found. Please ensure that either `bazel` or \
             `bazelisk` is installed and available in the `PATH`."
        )
    )]
    BazelNotFound,

    /// Error returned when `cargo` is not found in the `PATH`.
    #[error(
        "{} {} {}",
        TOOLCHAIN_ERROR_PREFIX.blue(),
        "-".black(),
        "cargo not found".red(),
    )]
    #[diagnostic(
        code(byakugan::toolchain::cargo_not_found),
        url(docsrs),
        help(
            "The `cargo` build tool was not found. Please ensure that `cargo` is installed and \
             available in the `PATH`."
        )
    )]
    CargoNotFound,
}
