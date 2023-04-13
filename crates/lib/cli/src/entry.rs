use clap::Parser;
use std::process::ExitCode;
use utils::Result;

use crate::ByakuganCli;

pub struct Byakugan;

impl Byakugan {
    /// Runs the **top level entry point** for **Byakugan** and returns an `ExitCode`
    /// indicating the success or failure of the program.
    pub fn run() -> Result<ExitCode> {
        // initialize the settings
        // let settings = LeafcSettings::new(&cli); // this api or
        // let settings = leafc_cfg::init_settings();
        // this in turn calls:

        // mutate the settings based on config files
        // leafc_cfg::update_settings_via_config_files(&mut settings);

        // parse the command line arguments
        let cli = ByakuganCli::parse();

        dbg!(cli);

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
