pub mod build_tools;

use build_tools::BuildSystem;
use cfg::settings::byakugan;
use clap::Parser;
use cli::{
    ByakuganCli,
    Command,
};
use derive_more::Display;
use getset::{
    Getters,
    MutGetters,
    Setters,
};
use miette::Result;
use owo_colors::OwoColorize;
use shrinkwraprs::Shrinkwrap;
use std::{
    process::ExitCode,
    time::Duration,
};
use typed_builder::TypedBuilder;
use utils::log;
use watch::Task;

use crate::build_tools::buck2;

/// State of watching a file system for changes.
#[derive(Debug, Clone, Display, Getters, MutGetters, Setters, TypedBuilder, Shrinkwrap)]
#[display(fmt = "Byakugan {{ settings: {settings} }}")]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Byakugan {
    settings: Settings,
}

#[derive(Debug, Clone, Display, Getters, MutGetters, Setters, TypedBuilder)]
#[display(fmt = "Settings {{ debounce_duration: {debounce_duration:?}, command: {command:?} }}")]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Settings {
    /// The duration to debounce file system events for (in milliseconds).
    /// [default: 1000]
    debounce_duration: Duration,

    /// The command to execute when a file system event is triggered.
    /// [default: `buck2|bazel build //...`]
    command: Box<dyn Task>,

    /// The build system to use.
    /// [default: detected from user's environment]
    build_system: BuildSystem,
    // Signals channel for the current process
    // signals: signals::Signals,
}

impl Byakugan {
    /// Create a new `Byakugan` instance with the default settings
    /// for the file system watcher except for those specified from
    /// the command line arguments or the configuration file.
    pub fn new(cli: ByakuganCli) -> Self {
        todo!()
        // let debounce_duration =
        // Duration::from_millis(cli.debounce_duration());

        // Self {
        //     debounce_duration,
        //     command: Box::new(cli.subcommand()),
        // }
    }

    /// Runs the **top level entry point** for **Byakugan** and returns an
    /// `ExitCode` indicating the success or failure of the program.
    pub fn run() -> Result<ExitCode> {
        // parse the command line arguments
        let command = ByakuganCli::parse();
        tracing::debug!("Canonical command issued: {}", command);
        // let state = Self::new(cli); // TODO: construct state from cli and config file

        log::init(command.verbosity())?;
        tracing::info!("{} is running", byakugan());

        // If the user specified a subcommand, then use that
        // otherwise, use the default subcommand `build`.
        let subcommand = if let Some(subcommand) = command.subcommand() {
            tracing::debug!("Subcommand specified: {}", subcommand);
            subcommand.clone()
        } else {
            tracing::debug!("No subcommand specified, using default: build");
            Command::Build(Default::default())
        };

        // Determine the build system to use.
        let build_system = build_tools::detect_build_system(&cli::str(&subcommand))?;
        tracing::info!("Build system detected: {build_system}");

        // Ensure that the build system is executable (i.e. it exists in the PATH)
        build_tools::ensure_build_system_executable(build_system)?;

        // Use the build system to validate that the targets are valid (i.e. they all
        // exist) and determine the task to invoke in watch mode.
        build_tools::validate_targets(&subcommand, build_system)?;

        // From this point on, we can assume that the build system is installed and
        // that the targets are valid, so we can safely execute the task.

        match subcommand {
            Command::Build(args) => {
                tracing::info!(
                    "Canonical command issued in {} mode: {}{} {}{}",
                    "WATCH".red().bold(),
                    "`".red(),
                    buck2(),
                    args,
                    "`".red()
                );
                tracing::info!(
                    "A {} mode build will be triggered anytime the transitive dependency\n\t      \
                     closure (tset) formed by the following targets changes:\n\n\t\t{}",
                    "WATCH".red().bold(),
                    args.targets()
                        .iter()
                        .map(|t| t.yellow().italic().to_string())
                        .collect::<Vec<_>>()
                        .join(&",\n\t\t".yellow().italic().to_string())
                );
                // watch::watch(build::ctx(args))?;
            }
            Command::Run(run) => {
                tracing::info!(
                    "Executing run command for {} given target {:?}",
                    build_system,
                    run.target()
                );
            }
            Command::Test(test) => {
                tracing::info!(
                    "Executing test command for {} targets {:?}",
                    build_system,
                    test.targets()
                );
            }
        }

        // dbg!(cli);

        // mutate the settings based on the command line arguments

        // leafc_cfg::update_settings_via_cli(&mut settings, &cli);

        // initialize the logging system
        // leafc_log::init(cli.verbosity)?;

        // log the settings

        // run the driver or repl as appropriate
        // if cli.sources().is_empty() {
        //     leafc_repl::entry(&cli)?;
        // } else {
        //     leafc_driver::batch_run(&cli)?;
        // }

        Ok(ExitCode::SUCCESS)
    }
}
