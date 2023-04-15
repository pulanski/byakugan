use std::fmt;

use cfg::settings::LogLevel;
use derivative::Derivative;
use diagnostics::errors::LogError;
use miette::{Diagnostic, IntoDiagnostic, ReportHandler, Result};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn init(verbosity: &LogLevel) -> Result<()> {
    // Initialize the logging subsystem
    let subscriber = FmtSubscriber::builder()
        .with_max_level(match verbosity {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
            LogLevel::Fatal => Level::ERROR,
        })
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .into_diagnostic()
        .map_err(|e| LogError::LogSystemInitialization(e.to_string().into()))?;

    // Initialize the error reporting subsystem
    miette::set_hook(Box::new(move |_| Box::new(ByakuganReportHandler::new()))).into_diagnostic()
}

// TODO: extend in the future

#[derive(Debug, Clone, Derivative)]
#[derivative(Default(new = "true"))]
pub struct ByakuganReportHandler;

impl ByakuganReportHandler {
    /// Render a [`Diagnostic`]. This function is mostly internal and meant to
    /// be called by the toplevel [`ReportHandler`] handler, but is made public
    /// to make it easier (possible) to test in isolation from global state.
    pub fn render_report(
        &self,
        f: &mut fmt::Formatter<'_>,
        diagnostic: &(dyn Diagnostic),
    ) -> fmt::Result {
        let mut diag = f.debug_struct("Diagnostic");
        diag.field("message", &format!("{}", diagnostic));
        // if let Some(code) = diagnostic.code() {
        //     diag.field("code", &code.to_string());
        // }
        // if let Some(severity) = diagnostic.severity() {
        //     diag.field("severity", &format!("{:?}", severity));
        // }
        // if let Some(url) = diagnostic.url() {
        //     diag.field("url", &url.to_string());
        // }
        // if let Some(help) = diagnostic.help() {
        //     diag.field("help", &help.to_string());
        // }
        // if let Some(labels) = diagnostic.labels() {
        //     let labels: Vec<_> = labels.collect();
        //     diag.field("labels", &format!("{:?}", labels));
        // }
        // if let Some(cause) = diagnostic.diagnostic_source() {
        //     diag.field("caused by", &format!("{:?}", cause));
        // }
        diag.finish()?;
        writeln!(f)
    }
}

impl ReportHandler for ByakuganReportHandler {
    fn debug(&self, diagnostic: &(dyn Diagnostic), f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return fmt::Debug::fmt(diagnostic, f);
        }

        self.render_report(f, diagnostic)
    }
}
