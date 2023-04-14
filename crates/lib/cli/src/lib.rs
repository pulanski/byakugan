mod entry;

mod flags;

use clap::{Args, Parser, Subcommand};
pub use entry::Byakugan;
use getset::Getters;
use smartstring::alias::String;

// use leafc_cfg::settings::{
//     meta::{pkg::EXE_NAME, version::LEAFC_VERSION},
//     EmitKind, LogLevel,
// };

/// **Command line interface** for **Byakugan**, a Rust-based file system watcher
/// written for Starlark-based build systems.
#[derive(Parser, Debug, Getters, PartialEq, Eq, Hash)]
#[command(
    author = "pulanski",
    version = "0.1.0",
    about = "Tool for file system watching in the context of build systems that allows you to monitor a directory and automatically trigger builds, tests, and other tasks when source code changes.",
    long_about = "A user-friendly file system watcher designed to simplify the process of automating your development workflow within the context of Starlark-based build systems such as Buck, Buck2, and Bazel.

Byakugan is specifically designed to work with Starlark-based build systems like Buck, Buck2, and Bazel, and comes equipped with an intuitive command-line interface for executing and managing builds, tests, and other tasks. For example, with Byakugan, you can easily a run web server binary (e.g. `buck2 run //:web-server` or `bazel run //:web-server`), as well as specify custom build options (e.g. `--cxxopt=-std=c++17` or `--config=debug`), and have Byakugan automatically re-build the server and restart the process such that the live server is always up-to-date with the latest changes to your codebase. This concept is sometimes referred to as 'hot reloading' or 'live reloading' in other contexts.",
    bin_name = "bkg"
)]
#[getset(get = "pub")]
pub struct ByakuganCli {
    /// The subcommand to execute
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Targets {
    targets: Vec<String>,
}

#[derive(Subcommand, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    #[clap(
        about = "Build a target in watch-mode (e.g. `buck2 build //backend/go/web-server:web-server` or `bazel build //...`).\nThe build command is re-run automatically when source code changes."
    )]
    Build(Build),
    #[clap(
        about = "Run a target in watch-mode (e.g. `buck2 run //backend/go/web-server:web-server` or `bazel run //...`).\nThe run command is re-run automatically when source code changes and the process is automatically restarted."
    )]
    Run(Run),
    #[clap(
        about = "Test a target in watch-mode (e.g. `buck2 test //backend/go/web-server:web-server` or `bazel test //...`).\nThe test command is re-run automatically when source code changes, with incremental test execution\nbeing performed and displayed in the terminal."
    )]
    Test(Test),
}

#[derive(Args, Debug, Default, PartialEq, Eq, Hash)]
pub struct Build {
    /// The targets to build (e.g. `//backend/go/web-server:web-server` or `//...`)
    #[arg(required = true)]
    targets: Vec<String>,
}

#[derive(Args, Debug, Default, PartialEq, Eq, Hash)]
pub struct Run {
    /// The target to run (e.g. `//backend/go/web-server:web-server`)
    #[arg(required = true)]
    targets: String,
}

#[derive(Args, Debug, Default, PartialEq, Eq, Hash)]
pub struct Test {}

// /// Duration of debounce (in milliseconds) for file system events
// /// (default: 1000)
// #[arg(short, long, default_value_t = 1000)]
// debounce: u64,

enum Flags {
    Bazel(BazelFlags),
    Buck2(Buck2Flags),
}

struct BazelFlags {}

struct Buck2Flags {}
