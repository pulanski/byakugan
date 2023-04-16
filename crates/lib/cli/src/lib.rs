mod flags;

use std::fmt;

use cfg::settings::{
    byakugan,
    LogLevel,
    EXE_ABOUT,
    EXE_AUTHOR,
    EXE_NAME,
    EXE_VERSION,
};
use clap::{
    Args,
    Parser,
    Subcommand,
};
use derive_more::Display;
use getset::Getters;
use smartstring::alias::String;

/// **Command line interface** for **Byakugan**, a Rust-based file system
/// watcher written for Starlark-based build systems.
#[derive(Parser, Debug, Getters, PartialEq, Eq, Hash)]
#[command(
    author = EXE_AUTHOR,
    version = EXE_VERSION,
    about = EXE_ABOUT,
    long_about = "A user-friendly file system watcher designed to simplify the process of automating your development workflow within the context of Starlark-based build systems such as Buck, Buck2, and Bazel, and hopefully other build systems in the future.

Byakugan is specifically designed to work with Starlark-based build systems like Buck, Buck2, and Bazel, and comes equipped with an intuitive command-line interface for executing and managing builds, tests, and other tasks. For example, with Byakugan, you can easily a run web server binary (e.g. `buck2 run //:web-server` or `bazel run //:web-server`), as well as specify custom build options (e.g. `--cxxopt=-std=c++17` or `--config=debug`), and have Byakugan automatically re-build the server and restart the process such that the live server is always up-to-date with the latest changes to your codebase. This concept is sometimes referred to as 'hot reloading' or 'live reloading' in other contexts.",
    bin_name = EXE_NAME,
)]
#[getset(get = "pub")]
// if subcommand is some, then display subcommand, otherwise display just
// byakugan #[display(
//     fmt = "{} {}",
//     "byakugan()",
//     "subcommand.as_ref().map(|s| s.to_string()).unwrap_or_else(||
// "".to_owned()).to_string()" )]
pub struct ByakuganCli {
    /// The subcommand to execute
    #[clap(subcommand)]
    subcommand: Option<Command>,

    /// The verbosity level to use for logging
    /// [default: info]
    #[clap(short = 'v', long, required = false, value_enum, default_value = "info")]
    pub verbosity: LogLevel,
}

impl fmt::Display for ByakuganCli {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{EXE_NAME}")?;
        match &self.subcommand {
            Some(subcommand) => write!(f, " {subcommand}"),
            None => Ok(()),
        }
    }
}

#[derive(Subcommand, Debug, Display, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    #[clap(about = "Build a target in watch-mode (e.g. `buck2 build \
                    //backend/go/web-server:web-server` or `bazel build //...`).\nThe build \
                    command is re-run automatically when source code changes.")]
    Build(Build),
    #[clap(about = "Run a target in watch-mode (e.g. `buck2 run \
                    //backend/go/web-server:web-server` or `bazel run //...`).\nThe run command \
                    is re-run automatically when source code changes and the process is \
                    automatically restarted.")]
    Run(Run),
    #[clap(about = "Test a target in watch-mode (e.g. `buck2 test \
                    //backend/go/web-server:web-server` or `bazel test //...`).\nThe test \
                    command is re-run automatically when source code changes, with incremental \
                    test execution\nbeing performed and displayed in the terminal.")]
    Test(Test),
}

pub fn str(cmd: &Command) -> String {
    match cmd {
        Command::Build(cmd) => cmd.to_string().into(),
        Command::Run(cmd) => cmd.to_string().into(),
        Command::Test(cmd) => cmd.to_string().into(),
    }
}

#[derive(Args, Debug, Display, Clone, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
#[display(fmt = "build {}", "targets.join(\" \")")]
pub struct Build {
    /// The targets to build (e.g. `//backend/go/web-server:web-server` or
    /// `//...`)
    #[arg(required = false, default_value = "//...")]
    pub targets: Vec<String>,
}

impl Default for Build {
    fn default() -> Self {
        Self { targets: vec!["//...".to_owned().into()] }
    }
}

#[derive(Args, Debug, Default, Display, Clone, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
#[display(fmt = "run {target}")]
pub struct Run {
    /// The target to run (e.g. `//backend/go/web-server:web-server`)
    #[arg(required = true)]
    pub target: String,
}

#[derive(Args, Debug, Default, Display, Clone, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
#[display(fmt = "test {}", "targets.join(\" \")")]
pub struct Test {
    /// The targets to test (e.g. `//backend/go/web-server:web-server` or
    /// `//...`)
    #[arg(required = false, default_value = "//...")]
    pub targets: Vec<String>,
}

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
