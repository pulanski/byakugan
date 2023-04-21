use clap::Parser;
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use miette::{
    miette,
    Result,
};
use std::{
    path::PathBuf,
    thread,
    time::Duration,
};
use tokio;
use tracing::{
    error,
    info,
};

mod extract_scene;

#[derive(Parser, Debug)]
#[clap(name = "Manim Rust")]
struct Opt {
    #[clap(short, long)]
    version: bool,

    #[clap(short, long)]
    // #[clap(short, long, parse(from_os_str))]
    file: Option<PathBuf>,
    // Additional CLI arguments here...
}

struct Driver;

impl Driver {
    async fn run() -> Result<()> {
        // Initialize tracing for diagnostics
        tracing_subscriber::fmt::init();

        let opt = Opt::parse();

        if opt.version && opt.file.is_none() {
            return Err(miette!("Version information is not available."));
        }

        // Load and parse configuration here...

        thread::sleep(Duration::from_secs(5));

        let scenes = extract_scene::get_scenes_to_render(/* Pass configuration here */);

        let pb = ProgressBar::new(scenes.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .expect("Failed to initialize progress bar UI.")
                .progress_chars("#>-"),
        );

        for scene in scenes {
            pb.inc(1);
            scene.run().await;
        }

        pb.finish_with_message("All scenes have been rendered.");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Driver::run().await
}
