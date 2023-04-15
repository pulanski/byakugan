use cli::Command;
use downcast_rs::Downcast;
use dyn_clone::DynClone;
use miette::{IntoDiagnostic, Result};
use notify::*;
// use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};
// use smartstring::alias::String;
use bytes::BytesMut;
use std::time::Duration;

/// A **task** that can be _executed_ and _monitored/manipulated_ by Byakugan.
pub trait Task: std::fmt::Debug + std::fmt::Display + Send + Sync + DynClone + Downcast {
    /// Start the task and return a `BytesMut` containing the output of the task.
    /// If the task is already running, this method should return an error.
    fn start(&self) -> Result<BytesMut>;

    /// Stop the task (e.g. by sending a `SIGTERM` signal to the process running the task).
    /// If the task is not running, this method should return an error.
    fn stop(&self) -> Result<()>;

    /// Kill the task (e.g. by sending a `SIGKILL` signal to the process running the task).
    /// If the task is not running, this method should return an error.
    fn kill(&self) -> Result<()>;

    /// Restart the task (e.g. by sending a `SIGKILL` signal to the process running the task).
    /// If the task is not running, this method should return an error.
    fn restart(&self) -> Result<()>;

    /// Get the current status of the task. This method should return a `BytesMut` containing the
    /// current status of the task (e.g. the output of the task).
    /// If the task is not running, this method should return an error.
    fn status(&self) -> Result<BytesMut>;

    /// Check if the task is running. This method should return `true` if the task is running,
    /// and `false` otherwise.
    fn is_running(&self) -> bool;
}

dyn_clone::clone_trait_object!(Task);

pub fn watch(command: Command) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    // This example is a little bit misleading as you can just create one Config and use it for all watchers.
    // That way the pollwatcher specific stuff is still configured, if it should be used.
    let mut watcher: Box<dyn Watcher> = if RecommendedWatcher::kind() == WatcherKind::PollWatcher {
        // custom config for PollWatcher kind
        // you
        let config = Config::default().with_poll_interval(Duration::from_secs(1));
        Box::new(PollWatcher::new(tx, config).unwrap())
    } else {
        // use default config for everything else
        Box::new(RecommendedWatcher::new(tx, Config::default()).unwrap())
    };

    // get the current directory
    let current_dir = std::env::current_dir().unwrap();

    // watch the current directory recursively
    watcher
        .watch(&current_dir, RecursiveMode::Recursive)
        .unwrap();

    tracing::info!("Watching current directory...");

    // ensure the validity of the command
    validate_command(&command)?;

    for e in rx {
        match e {
            Ok(event) => match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => match command
                {
                    Command::Build(ref args) => {
                        tracing::info!("update detected, rebuilding... {:?}", args);

                        // todo, figure out which build system we're during invocation, and then execute the appropriate command
                        // this should be done via loading it into the context, and then using the context
                        // behind the build command

                        let mut cmd = std::process::Command::new("buck2");
                        cmd.arg("build");
                        for arg in args.targets() {
                            cmd.arg(arg.to_string());
                        }

                        cmd.stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .spawn()
                            .into_diagnostic()?;
                    }
                    Command::Run(_) => {
                        tracing::info!("update detected, rebuilding and restarting process...");
                    }
                    Command::Test(_) => {
                        tracing::info!("update detected, rebuilding and re-executing tests...");
                    }
                },
                EventKind::Any | EventKind::Access(_) | EventKind::Other => {}
            },
            Err(e) => tracing::error!("watch error: {}", e),
        }
    }

    Ok(())
}

fn validate_command(command: &Command) -> Result<()> {
    match command {
        Command::Build(args) => validate_build_command(args),
        Command::Run(_) => Ok(()),
        Command::Test(_) => Ok(()),
    }
}

fn validate_build_command(args: &cli::Build) -> Result<()> {
    // ensure the targets specified are valid (i.e. they exist)
    // for target in args.targets() {
    //     if !target.exists() {
    //         return Err(format!("target `{}` does not exist", target.display()).into());
    //     }
    // }

    Ok(())
}
