pub mod build_tools;
use build_tools::context::build;
use clap::Parser;

use cfg::settings::byakugan;
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
use shrinkwraprs::Shrinkwrap;
use std::{
    process::ExitCode,
    time::Duration,
};
use typed_builder::TypedBuilder;
use utils::log;
use watch::Task;

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

    command: Box<dyn Task>,
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

        // Determine the build system to use.
        let subcommand = command.subcommand();
        let build_system = build_tools::detect_build_system(&cli::str(subcommand))?;
        tracing::info!("Build system detected: {build_system}");

        // Ensure that the build system is executable (i.e. it exists in the PATH)
        build_tools::ensure_build_system_executable(build_system)?;

        // Use the build system to validate that the targets are valid (i.e. they all
        // exist) and determine the task to invoke in watch mode.
        build_tools::validate_targets(command.subcommand(), build_system)?;

        // if valid_targets.is_empty() {
        //     tracing::warn!("No valid targets were found");
        //     return Ok(ExitCode::SUCCESS);
        // }

        match command.subcommand() {
            Command::Build(args) => {
                tracing::info!(
                    "Executing build command for {} given targets {:?}",
                    build_system,
                    args.targets()
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
