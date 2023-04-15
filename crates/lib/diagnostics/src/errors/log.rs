use miette::Diagnostic;
use owo_colors::OwoColorize;
use smol_str::SmolStr;
use thiserror::Error;

/// Prefix appended to all `LogError` messages.
pub const LOG_ERROR_PREFIX: &str = "Logging System Error";

#[derive(Debug, Error, Diagnostic, Clone)]
pub enum LogError {
    /// Error returned when the **log file** could not be **initialized**.
    #[error(
        "{} {} {}{} {}",
        LOG_ERROR_PREFIX.blue(),
        "-".black(),
        "Log file initialization failed".red(),
        ":".black(),
        .0.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::log::log_file_initialization),
        url(docsrs),
        help(
            "The log file could not be initialized. Please try again (and report this issue if it \
             persists)."
        )
    )]
    LogFileInitialization(SmolStr),

    /// This error is returned when an **error occurs** during the
    /// **initialization** of the logging system.
    #[error(
        "{} {} {}{} {}",
        LOG_ERROR_PREFIX.blue(),
        "-".black(),
        "Log system initialization failed".red(),
        ":".black(),
        .0.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::log::log_system_initialization),
        url(docsrs),
        help(
            "The log system could not be initialized. Please try again (and report this issue if \
             it persists)."
        )
    )]
    LogSystemInitialization(SmolStr),

    /// This error is returned when the **log file** could not be **opened**.
    /// This error is typically returned during the **initialization** of the
    /// logging system.
    #[error(
        "{} {} {}{} {}",
        LOG_ERROR_PREFIX.blue(),
        "-".black(),
        "Log file could not be opened".red(),
        ":".black(),
        .0.yellow().italic()
    )]
    #[diagnostic(
        code(byakugan::log::log_file),
        url(docsrs),
        help(
            "The log file could not be opened. Please try again (and report this issue if it \
             persists)."
        )
    )]
    LogFileOpen(SmolStr),
}
