mod bazel;
mod buck2;
mod cargo;

use std::{
    env,
    path::{Path, PathBuf},
};

use super::label::Label;
use derive_more::Display;
use diagnostics::errors::CliError::NoBuildSystemDetected;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use smartstring::alias::String;

fn buck2() -> String {
    "buck2".bright_yellow().bold().to_string().into()
}

fn bazel() -> String {
    "bazel".bright_green().bold().to_string().into()
}

fn cargo() -> String {
    "cargo".yellow().bold().to_string().into()
}

/// A **build system** used to _execute a build command_. Used to determine which build system to use
/// when executing a build command (e.g. `buck2`, `bazel`, `cargo`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{}")]
pub enum BuildSystem {
    #[display(fmt = "{}", "buck2()")]
    Buck,
    Bazel,
    Cargo,
    // ... more build systems in the future potentially (e.g. make, ninja, etc.)
}

enum TaskContext {
    Build(BuildContext),
    // ... more contexts in the future potentially (e.g. test, run, etc.)
}

/// The **context** in which a build command is executed. The context contains relevant information about
/// the build command, the build system, and the build targets that are being built.
struct BuildContext {
    /// The **targets** that are being built by the build command (e.g. `@fbcode//foo/bar:baz` in
    /// `buck2 build @fbcode//foo/bar:baz` or `...` in `bazel build ...`)
    pub targets: Vec<Target>,

    /// The **build system** used to execute the build command (e.g. `buck2`, `bazel`, `cargo`, etc.).
    ///
    /// **NOTE**: This is used to determine which build system to use when executing
    /// a build command and need not be specified explicitly by the user as it can be detected from
    /// a combination of the build command and the presence of a build system configuration file
    /// (e.g. `buckconfig`, `WORKSPACE`, `Cargo.toml`, etc.).
    pub system: BuildSystem,
}

/// A **build target**. A build target is a label that identifies a **build target**
/// in the context of a **build system**. A build target is composed of a **label**
/// (e.g. `@fbcode//foo/bar:baz`) and a **rule** (e.g. `rust_binary`) used to build the target.
///
/// **NOTE**: At this time targets are only used when executing commands in the context of a
/// Starlark-based build system (e.g. **Buck(2)**, **Bazel**, etc.), however in the future targets
/// may be made more generic to support other build systems (e.g. **Cargo**, **Make**, **Ninja**, etc.),
/// or another abstraction may be used to represent build targets in a more generic way.
struct Target {
    label: Label,
    rule: Rule,
}

struct Rule {
    // ...
}

/// Detect build system by searching for a build system configuration file in the current directory
/// or any of its parent directories, searching recursively up to the root directory. If a
/// configuration file is found, the corresponding build system is used to execute the build command,
/// otherwise an error is returned.
///
/// **NOTE**: There is a hierarchy of build system configuration files that is used to determine
/// the precedence of build systems when multiple build system configuration files are found in the
/// current directory or any of its parent directories. The hierarchy in order of precedence from
/// highest to lowest is:
///
/// 1. `.buckconfig` (Buck(2))
/// 2. `WORKSPACE` | `WORKSPACE.bazel` (Bazel)
/// 3. `Cargo.toml` (Cargo)
///
/// # Arguments
///
/// * `cmd` - The canonical command issued by the user (e.g. `bkg build //foo/bar:baz`).
///           This is used in the error message when no build system configuration file is found.
#[tracing::instrument]
pub fn detect_build_system(cmd: &str) -> Result<BuildSystem> {
    let current_dir = env::current_dir().into_diagnostic()?;

    // Search for build system configuration files in current directory and its parent directories
    let mut dir = PathBuf::from(&current_dir);
    loop {
        if let Some(build_system) = search_for_build_system(&dir) {
            return Ok(build_system);
        }

        if !dir.pop() {
            break; // Reached the root directory
        }
    }

    Err(NoBuildSystemDetected {
        command: cmd.to_string(),
    })
    .into_diagnostic()
}

#[tracing::instrument]
fn search_for_build_system(dir: &Path) -> Option<BuildSystem> {
    let buck_config = dir.join(".buckconfig");
    if buck_config.exists() {
        return Some(BuildSystem::Buck);
    }

    let workspace_bazel = dir.join("WORKSPACE.bazel");
    let workspace = dir.join("WORKSPACE");
    if workspace_bazel.exists() || workspace.exists() {
        return Some(BuildSystem::Bazel);
    }

    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        return Some(BuildSystem::Cargo);
    }

    None // No build system configuration file found in this directory
}

pub mod build {
    use cli::{Build, Command};

    pub fn ctx(args: &Build) -> Command {
        // pub fn ctx(args: &Build) -> BuildContext {
        // let mut ctx = BuildContext::default();
        // ...
        // ctx.targets = args.targets().to_vec();
        // ...
        // ctx

        Command::Build(args.clone())
    }
}

/// Validate that the targets specified by the user are valid for the given build system.
/// If the targets are valid, return `Ok(true)`, otherwise return `Ok(false)`.
/// If an error occurs, return `Err`.
///
/// **NOTE**: This is a temporary solution to validate targets for both Buck(2) and Bazel.
/// In the future, this will be replaced by a more generic solution that can be used to validate
/// targets for any build system.
pub(crate) fn validate_targets(subcommand: &cli::Command, build_system: BuildSystem) -> Result<()> {
    // Use the detected build system to validate the targets specified by the user
    match build_system {
        BuildSystem::Buck => {
            // Validate targets for Buck(2)

            // Use `buck2 query` to collect all build targets in the current workspace/cell
            let all_targets = buck2::query::all_targets()?;

            Ok(())
        }
        BuildSystem::Bazel => {
            // Validate targets for Bazel
            // ...

            todo!("Validate targets for Bazel")
        }
        BuildSystem::Cargo => {
            // Validate targets for Cargo
            // ...

            todo!("Validate targets for Cargo")
        }
    }
}
