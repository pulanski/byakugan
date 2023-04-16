mod bazel;
mod buck2;
mod cargo;

use std::{
    collections::BinaryHeap,
    env,
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};

use super::label::Label;
use derivative::Derivative;
use derive_more::Display;
use diagnostics::errors::{
    CliError::NoBuildSystemDetected,
    ToolchainError,
};
use dirs_next::cache_dir;
use getset::{
    Getters,
    MutGetters,
    Setters,
};
use miette::miette;
use miette::{
    IntoDiagnostic,
    Report,
    Result,
    SourceSpan,
};
use owo_colors::OwoColorize;
use shrinkwraprs::Shrinkwrap;
use smartstring::alias::String;
use strsim::levenshtein;
use typed_builder::TypedBuilder;

pub fn buck2() -> String {
    "buck2".bright_yellow().bold().to_string().into()
}

pub fn bazel() -> String {
    "bazel".bright_green().bold().to_string().into()
}

pub fn cargo() -> String {
    "cargo".yellow().bold().to_string().into()
}

/// A **build system** used to _execute a build command_. Used to determine
/// which build system to use when executing a build command (e.g. `buck2`,
/// `bazel`, `cargo`, etc.).
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

/// The **context** in which a build command is executed. The context contains
/// relevant information about the build command, the build system, and the
/// build targets that are being built.
struct BuildContext {
    /// The **targets** that are being built by the build command (e.g.
    /// `@fbcode//foo/bar:baz` in `buck2 build @fbcode//foo/bar:baz` or
    /// `...` in `bazel build ...`)
    pub targets: TargetSet,

    /// The **build system** used to execute the build command (e.g. `buck2`,
    /// `bazel`, `cargo`, etc.).
    ///
    /// **NOTE**: This is used to determine which build system to use when
    /// executing a build command and need not be specified explicitly by
    /// the user as it can be detected from a combination of the build
    /// command and the presence of a build system configuration file
    /// (e.g. `buckconfig`, `WORKSPACE`, `Cargo.toml`, etc.).
    pub system: BuildSystem,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Getters,
    MutGetters,
    Setters,
    TypedBuilder,
    Derivative,
    Shrinkwrap,
)]
#[derivative(Default(new = "true"))]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
#[shrinkwrap(mutable)]
struct TargetSet {
    targets: Vec<Target>,
}

// impl display
impl std::fmt::Display for TargetSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let targets = self
            .targets
            .iter()
            .map(|target| target.label().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{targets}")
    }
}

impl TargetSet {
    /// Get the **target** that matches the specified **label** from the
    /// **target set**. If no target is found, an error is returned
    /// (indicating that the target was not found within the dependency graph of
    /// the build system).
    ///
    /// # Arguments
    ///
    /// * `cmd` - The canonical command executed by the user (e.g. `buck2 build
    ///  @fbcode//foo/bar:baz` or `bazel build @fbcode//foo/bar:baz`). This is
    /// used to generate a helpful error message if the target is not found.
    ///
    /// * `label` - The label of the target to get from the target set.
    ///
    /// # Returns
    ///
    /// * `Ok(&Target)` - The target that matches the specified label.
    ///
    /// * `Err(Report)` - An error indicating that the target was not found
    /// within the dependency graph of the build system.
    pub fn get_target(&self, cmd: &str, label: &Label) -> Result<&Target> {
        // TODO: in the future, calculate the span of the label in the command correctly
        // self.targets.iter().find(|target| target.label() == label).ok_or_else(|| {
        //     Report::new(CliError::TargetNotFound {
        //         command: cmd.to_string(),
        //         target:  label.to_string().into(),
        //         span:    SourceSpan::new(0.into(), label.to_string().len().into()),
        //     })
        // })
        todo!()
    }
}

impl From<Vec<String>> for TargetSet {
    fn from(targets: Vec<String>) -> Self {
        Self {
            targets: targets
                .into_iter()
                .map(|target| {
                    Target::builder()
                        .label(Label::from(target.as_str()))
                        .rule(Rule::default())
                        .build()
                })
                .collect(),
        }
    }
}

/// A **build target**. A build target is a label that identifies a **build
/// target** in the context of a **build system**. A build target is composed of
/// a **label** (e.g. `@fbcode//foo/bar:baz`) and a **rule** (e.g.
/// `rust_binary`) used to build the target.
///
/// **NOTE**: At this time targets are only used when executing commands in the
/// context of a Starlark-based build system (e.g. **Buck(2)**, **Bazel**,
/// etc.), however in the future targets may be made more generic to support
/// other build systems (e.g. **Cargo**, **Make**, **Ninja**, etc.), or another
/// abstraction may be used to represent build targets in a more generic way.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, MutGetters, Setters, TypedBuilder)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
struct Target {
    label: Label,
    rule:  Rule,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Getters, MutGetters, Setters, TypedBuilder, Derivative,
)]
#[derivative(Default(new = "true"))]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
struct Rule {
    // ...
}

/// Detect build system by searching for a build system configuration file in
/// the current directory or any of its parent directories, searching
/// recursively up to the root directory. If a configuration file is found, the
/// corresponding build system is used to execute the build command,
/// otherwise an error is returned.
///
/// **NOTE**: There is a hierarchy of build system configuration files that is
/// used to determine the precedence of build systems when multiple build system
/// configuration files are found in the current directory or any of its parent
/// directories. The hierarchy in order of precedence from highest to lowest is:
///
/// 1. `.buckconfig` (Buck(2))
/// 2. `WORKSPACE` | `WORKSPACE.bazel` (Bazel)
/// 3. `Cargo.toml` (Cargo)
///
/// # Arguments
///
/// * `cmd` - The canonical command issued by the user (e.g. `bkg build
///   //foo/bar:baz`). This is used in the error message when no build system
///   configuration file is found.
#[tracing::instrument]
pub fn detect_build_system(cmd: &str) -> Result<BuildSystem> {
    let current_dir = env::current_dir().into_diagnostic()?;

    // Search for build system configuration files in current directory and its
    // parent directories
    let mut dir = PathBuf::from(&current_dir);
    loop {
        if let Some(build_system) = search_for_build_system(&dir) {
            return Ok(build_system);
        }

        if !dir.pop() {
            break; // Reached the root directory
        }
    }

    Err(NoBuildSystemDetected { command: cmd.to_string() }).into_diagnostic()
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
    use cli::{
        Build,
        Command,
    };

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

/// Validate that the targets specified by the user are valid for the given
/// build system. If the targets are not valid, an error is returned.
///
/// **NOTE**: This is a temporary solution to validate targets for both Buck(2)
/// and Bazel. In the future, this will be replaced by a more generic solution
/// that can be used to validate targets for any build system.
pub(crate) fn validate_targets(subcommand: &cli::Command, build_system: BuildSystem) -> Result<()> {
    // Use the detected build system to validate the targets specified by the user
    match build_system {
        BuildSystem::Buck => {
            // Validate targets for Buck(2)

            // Use `buck2 query` to collect all build targets in the current workspace/cell
            let all_targets = buck2::query::all_targets()?;

            // Check that all targets specified by the user exist within the target list
            // collected
            match subcommand {
                cli::Command::Build(cli::Build { targets }) |
                cli::Command::Test(cli::Test { targets }) => {
                    // TODO:
                    // let requested_targets = TargetSet::from(targets.clone());
                    // tracing::debug!("Validating targets: {}",
                    // requested_targets);

                    // temporary workaround
                    if targets.len() == 1 && targets[0] == "//..." {
                        tracing::debug!("All targets requested...");
                        return Ok(());
                    }

                    let mut did_you_mean = BinaryHeap::new();
                    let mut invalid_targets = Vec::new();

                    for target in targets {
                        tracing::debug!(
                            "Validating target: {}",
                            &target.trim().to_string().yellow()
                        );

                        let all_targets = all_targets
                            .iter()
                            .map(|t| t.split("//").collect::<Vec<&str>>())
                            .map(|t| t[t.len() - 1])
                            .collect::<Vec<&str>>();

                        let target = if target.contains("//") {
                            // get the last part of the target
                            let target = target.split("//").collect::<Vec<&str>>();
                            target[target.len() - 1]
                        } else {
                            target
                        };

                        if !all_targets.contains(&target.trim().to_string().as_str())
                        // if !all_targets.contains(&target.trim().to_string()) ||
                        //     // or all targets split // is not in the list
                        //     // TODO:
                        {
                            invalid_targets.push(target.trim().to_string());
                            tracing::debug!("Invalid target: {}", &target.trim().to_string().red());
                            // Find the top 5 closest matches to the invalid target using
                            // Levenshtein distance
                            for valid_target in all_targets {
                                let distance = levenshtein(target, valid_target);
                                tracing::trace!(
                                    "Distance between '{}' and '{}' is {}",
                                    target,
                                    valid_target,
                                    distance
                                );
                                if did_you_mean.len() < 5 {
                                    tracing::trace!("Pushing to heap");
                                    did_you_mean.push((distance, valid_target));
                                } else if let Some((current_max_distance, _)) = did_you_mean.peek()
                                {
                                    if distance < *current_max_distance {
                                        tracing::trace!(
                                            "Found a closer match, popping top of heap and \
                                             pushing new match"
                                        );
                                        did_you_mean.pop();
                                        did_you_mean.push((distance, valid_target));
                                    }
                                }
                            }
                        }
                    }

                    // Construct the DidYouMean error message with the top 5
                    // closest matches
                    let mut closest_matches = vec![];
                    while let Some((_, target)) = did_you_mean.pop() {
                        closest_matches.push(target);
                    }
                    closest_matches.reverse();

                    if !invalid_targets.is_empty() {
                        println!(
                            "\n{}\n",
                            "Did you mean one of the following targets instead?".blue()
                        );
                        for closest_match in &closest_matches {
                            println!("  {}", closest_match.green());
                        }
                        println!();

                        return Err(miette!(
                            "Invalid target(s) specified\n\n{}",
                            invalid_targets
                                .iter()
                                .map(|t| t.yellow().italic().to_string())
                                .collect::<Vec<_>>()
                                .join(&",\n".yellow().italic().to_string())
                        ));
                    }

                    // TODO: refactor to this in future potentially, when fancy
                    // feature in miette works properly for
                    // buck2

                    // if invalid_target_count > 0 {
                    //     if invalid_target_count == 1 {
                    //         return Err(TargetNotFound {
                    //             command:      subcommand.to_string(),
                    //             target:       targets[0].to_string().into(),
                    //             did_you_mean: closest_matches
                    //                 .into_iter()
                    //                 .map(|t| t.to_string())
                    //                 .collect(),
                    //             span:         SourceSpan::new(
                    //                 0.into(),
                    //                 subcommand.to_string().len().into(),
                    //             ),
                    //         })
                    //         .into_diagnostic();
                    //     } else {
                    //         return Err(TargetsNotFound {
                    //             targets:      targets.iter().map(|t|
                    // t.to_string()).collect(),
                    // command:      subcommand.to_string(),
                    //             span:         SourceSpan::new(
                    //                 0.into(),
                    //                 subcommand.to_string().len().into(),
                    //             ),
                    //             did_you_mean: closest_matches
                    //                 .into_iter()
                    //                 .map(|t| t.to_string())
                    //                 .collect(),
                    //         })
                    //         .into_diagnostic();
                    //     }
                    // }
                }
                cli::Command::Run(cli::Run { target }) => {
                    tracing::debug!("Validating target: {:?}", target);
                    // if !all_targets.contains(target) {
                    //     return Err(InvalidTarget { target: target.to_string()
                    // }).into_diagnostic(); }
                }
            }

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

pub(crate) fn ensure_build_system_executable(build_system: BuildSystem) -> Result<()> {
    match build_system {
        BuildSystem::Buck => {
            // Ensure that `buck2` is installed and available on the `PATH`

            if buck2::is_installed() {
                Ok(())
            } else {
                Err(ToolchainError::Buck2NotFound).into_diagnostic()
            }
        }
        BuildSystem::Bazel => {
            // Ensure that `bazel` is installed and available on the `PATH`

            if bazel::is_installed() {
                Ok(())
            } else {
                Err(ToolchainError::BazelNotFound).into_diagnostic()
            }
        }
        BuildSystem::Cargo => {
            // Ensure that `cargo` is installed and available on the `PATH`

            if cargo::is_installed() {
                Ok(())
            } else {
                Err(ToolchainError::CargoNotFound).into_diagnostic()
            }
        }
    }
}

/// Check if the given binary is installed and available on the `PATH`.
pub fn is_binary_installed(binary: &str) -> bool {
    // Check to see if a cached value exists for this check
    if let Some(is_installed) = cached_binary_install(binary) {
        if is_installed {
            tracing::debug!("Found cached {} install, skipping runtime check", binary);
            return true;
        }
    }

    // At this point, we know that the cached value is either not set or is set
    // to false. We need to check if the binary is installed and available on the
    // `PATH`.

    let is_installed = check_binary_install(binary);

    // Update the cache with the result of the check
    update_binary_install_cache(binary, is_installed)
        .expect("Unable to update binary install cache");

    is_installed
}

fn update_binary_install_cache(binary: &str, is_installed: bool) -> Result<()> {
    let install_cache_path = binary_install_cache_path(binary);

    if let Some(parent) = install_cache_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }
    }

    if !install_cache_path.exists() {
        // Cache file does not exist, so we need to create it
        match create_binary_install_cache(binary) {
            Ok(_) => {
                tracing::debug!("Created {} install cache file", binary);
            }
            Err(e) => {
                tracing::warn!("Unable to create bazel install cache file: {}", e);
                return Err(e);
            }
        }
    }

    fs::write(&install_cache_path, is_installed.to_string()).into_diagnostic().map(|_| ())
}

fn check_binary_install(binary: &str) -> bool {
    Command::new(binary).arg("--version").output().is_ok()
}

fn cached_binary_install(binary: &str) -> Option<bool> {
    let install_cache_path = binary_install_cache_path(binary);

    if install_cache_path.exists() {
        if let Ok(install_cache) = fs::read_to_string(&install_cache_path) {
            if let Ok(is_installed) = install_cache.parse::<bool>() {
                return Some(is_installed);
            }
        }

        tracing::warn!("Unable to read cached binary install value for {}, removing cache", binary);
        fs::remove_file(&install_cache_path).expect("Unable to remove binary install cache");
        None
    } else {
        // Cache file does not exist, so we need to create it
        match create_binary_install_cache(binary) {
            Ok(_) => Some(false),
            Err(e) => {
                tracing::warn!("Unable to create binary install cache: {}", e);
                None
            }
        }
    }
}

fn create_binary_install_cache(binary: &str) -> Result<()> {
    let install_cache_path = binary_install_cache_path(binary);

    // Create the cache directory if it does not exist
    if let Some(parent) = install_cache_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }
    }

    if !install_cache_path.exists() {
        fs::File::create(&install_cache_path).into_diagnostic()?;
    }

    Ok(())
}

fn binary_install_cache_path(binary_name: &str) -> PathBuf {
    let cache_dir = cache_dir().expect("Unable to determine cache directory");

    cache_dir.join("byakugan").join(format!("{binary_name}.installed"))
}
