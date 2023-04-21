#![feature(const_trait_impl)]

mod cache;
mod diagnostics;

use anyhow::Result;
use cargo_toml::{
    DepsSet,
    Manifest,
};
use cfg::settings::{
    EXE_AUTHOR,
    EXE_VERSION,
};
use clap::Parser;
use derive_more::Display;
use diagnostics::RudolphError;
use fxhash::FxHasher;
use getset::{
    Getters,
    MutGetters,
    Setters,
};
use humansize::{
    format_size,
    BINARY,
};
use owo_colors::OwoColorize;
use redis::{
    Client,
    Connection,
};
use std::collections::HashMap;
use std::fs::File;
use std::hash::{
    Hash,
    Hasher,
};
use std::io::{
    self,
    BufRead,
    BufReader,
    Write,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Stdio,
};
use std::time::SystemTime;
use tracing_subscriber::{
    EnvFilter,
    FmtSubscriber,
};
use typed_builder::TypedBuilder;

pub const RUDOLPH_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const EXE_NAME: &str = env!("CARGO_PKG_NAME");
pub const EXE_ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

// there is a rudolph.toml file which contains configuration for
// the data structure detailed below

#[derive(Display, Getters, MutGetters, Setters, TypedBuilder)]
#[display(fmt = "rudolph (v{RUDOLPH_VERSION})")]
pub struct Rudolph {
    /// The path to the third-party directory from which
    /// third-party Rust crates will be vendored and buckified.
    /// [default: `third-party`]
    third_party_dir: PathBuf,

    /// Whether or not to use the embedded reindeer binary
    /// to vendor and buckify third-party Rust crates.
    /// [default: true]
    embed: bool,

    // The path to a user-provided reindeer binary
    // If this is None, then the embedded reindeer binary
    // will be used.
    reindeer: Option<PathBuf>,

    /// The multi-level cache to use for caching third-party
    /// Rust crates.
    cache: Cache,
}

#[derive(Debug, Clone, Display, Getters, MutGetters, Setters, TypedBuilder, Parser)]
#[display(fmt = "rudolph_cli (v{RUDOLPH_VERSION})")]
#[command(
    author = EXE_AUTHOR,
    version = EXE_VERSION,
    about = EXE_ABOUT,
    bin_name = EXE_NAME,
)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct RudolphCli {
    /// The path to the third-party directory from which
    /// third-party Rust crates will be vendored and buckified.
    /// [default: `third-party`]
    #[clap(short, long, default_value = "third-party", required = false)]
    third_party_dir: PathBuf,

    /// Whether or not to use the embedded reindeer binary
    /// to vendor and buckify third-party Rust crates.
    /// [default: true]
    #[clap(short, long, default_value = "true", required = false)]
    embed: bool,

    // The path to a user-provided reindeer binary
    // If this is None, then the embedded reindeer binary
    // will be used.
    #[clap(short, long, default_value = None, required = false)]
    reindeer: Option<PathBuf>,

    /// Whether or not to ignore the cache and re-vendor
    /// and re-buckify third-party Rust crates.
    /// [default: false]
    #[clap(short, long, default_value = "false", required = false)]
    force: bool,
}

pub struct CacheConfig {
    /// The path to the cache directory.
    /// [default: `~/.rudolph/cache`] (or equivalent)
    path: PathBuf,
}

pub struct CacheState {
    /// A serialized hash of the most up-to-date version of the dependency set
    /// of third-party Rust crates used throughout the workspace as defined
    /// by the `Cargo.toml` file in the third-party directory.
    ///
    /// **NOTE**: This is only constructed in the event that the latest
    /// modification time of the Cargo.toml file in the third-party
    /// directory is greater than the latest modification time of the
    /// Cargo.toml file in either the warm cache or the cold cache.
    /// Otherwise, the cache is up-to-date and no work needs to be done.
    latest_deps_set_hash: HashMap<PathBuf, Vec<u8>>,

    /// The last modified time of the Cargo.toml file in the third-party
    /// directory.
    latest_cargo_toml_mod_time: HashMap<PathBuf, SystemTime>,
    // The in-memory Redis cache used to store the
    // warm cache.
    // redis: Connection,
    // The cold cache is a cached file system snapshot
    // of the third-party directory. This is used whenever
    // the warm cache is not available.
    // fs:      File,
}

impl From<&PathBuf> for CacheState {
    fn from(path: &PathBuf) -> Self {
        Self {
            latest_deps_set_hash:       HashMap::new(),
            latest_cargo_toml_mod_time: HashMap::new(),
            // redis: Connection::open("redis://
            // fs:      File::open(version),
        }
    }
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            latest_deps_set_hash:       HashMap::new(),
            latest_cargo_toml_mod_time: HashMap::new(),
            // redis: Connection::open("redis://
            // fs:      File::open(version),
        }
    }
}

/// A multi-level cache for third-party Rust crates.
/// In general, the cache is organized as follows:
///
/// The version of the third-party Rust crates used
/// throughout the workspace is kept up-to-date based
/// on the last modified time of the `Cargo.toml` file
/// in the third-party directory, i.e. the `Cargo.toml`
/// file in the third-party directory is the source of
/// truth for the version of the third-party Rust crates
/// used throughout the workspace.
///
/// The cache is organized as follows:
///
/// 1. The cold cache is a cached file system snapshot
///   of the third-party directory. This is used to
///  determine if the third-party directory has changed
/// since the last time the cache was updated.
/// 2. The warm cache is a cached in-memory representation
///  of the third-party directory. This is used to
/// determine if the third-party directory has changed
/// since the last time the cache was updated, but is
/// faster than the cold cache.
#[derive(Display, Getters, MutGetters, Setters, TypedBuilder)]
#[display(fmt = "rudolph_cache (v{RUDOLPH_VERSION})")]
pub struct Cache {
    /// The configuration for the cache.
    config: CacheConfig,
    /// The up-to-date state of the cache.
    /// Updated only when the cache is invalidated.
    state:  CacheState,
}

impl From<&PathBuf> for Cache {
    fn from(third_party_dir: &PathBuf) -> Self {
        let config = CacheConfig { path: get_cache_dir() };
        let state = CacheState::from(third_party_dir);
        Self { config, state }
    }
}

impl Cache {
    // pub fn from(third_party_dir: &Path) -> Self {
    //     let config = CacheConfig { path: get_cache_dir() };
    //     let version =
    // get_version_from_cargo_toml(&third_party_dir.join("Cargo.toml"));     let
    // state = CacheState::from(third_party_dir);     Self { config, state }
    // }

    // pub fn is_up_to_date(&self) -> bool {
    //     // Short-circuiting check to see if the cache is up-to-date
    //     // based on the last modified time of the Cargo.toml file
    //     // and then the hash of the dependency set of third-party
    //     // Rust crates used throughout the workspace.
    //     self.check_cargo_toml_mod_time() || self.check_deps_set()
    // }

    // pub fn check_cargo_toml_mod_time(&self) -> bool {
    //     self.latest_recorded_cargo_toml_mod_time() ==
    // self.current_cargo_toml_mod_time() }

    // pub fn latest_recorded_cargo_toml_mod_time(&self) -> SystemTime {
    //     self.state.latest_cargo_toml_mod_time
    // }

    // pub fn current_cargo_toml_mod_time(&self) -> SystemTime {
    //     let path = &self.config.path;
    //     let path = path.join("Cargo.toml");
    //     let metadata = fs::metadata(
    //     metadata.modified().expect("Could not read Cargo.toml")
    // }

    // pub fn check_deps_set(&self) -> bool {
    //     self.latest_recorded_deps_set() == self.current_deps_set()
    // }

    // pub fn latest_recorded_deps_set(&self) -> String {
    //     self.state.latest_deps_set_hash.iter().map(|b| *b as char).collect()
    // }

    pub fn current_deps_set(&self) -> String {
        get_version_from_cargo_toml(&self.config.path)
    }
}

fn get_version_from_cargo_toml(path: &PathBuf) -> String {
    let manifest = Manifest::from_path(path).expect("Could not read Cargo.toml");

    // Dependencies
    let deps = manifest.dependencies;
    let mut deps = deps.iter().collect::<Vec<_>>();
    deps.sort_by(|a, b| a.0.cmp(b.0));

    let mut hasher = FxHasher::default();
    for (name, dep) in deps {
        name.hash(&mut hasher);
        dep.detail().expect("Could not get dependency detail").version.hash(&mut hasher);
    }

    // Dev dependencies
    let dev_deps = manifest.dev_dependencies;
    let mut dev_deps = dev_deps.iter().collect::<Vec<_>>();
    dev_deps.sort_by(|a, b| a.0.cmp(b.0));

    for (name, dep) in dev_deps {
        name.hash(&mut hasher);
        dep.detail().expect("Could not get dependency detail").version.hash(&mut hasher);
    }

    // Build dependencies
    let build_deps = manifest.build_dependencies;
    let mut build_deps = build_deps.iter().collect::<Vec<_>>();
    build_deps.sort_by(|a, b| a.0.cmp(b.0));

    for (name, dep) in build_deps {
        name.hash(&mut hasher);
        dep.detail().expect("Could not get dependency detail").version.hash(&mut hasher);
    }

    let version = hasher.finish().to_string();
    tracing::debug!("Version of third-party Rust crates: {}", version);

    version
}

fn get_fs_cache(path: &Path) -> File {
    let mut path = path.to_path_buf();
    path.push("fs.version");
    tracing::trace!("Opening fs-based \"cold\" cache at {:?}", path);

    // If the filesystem cache does not exist, create it.
    if !path.exists() {
        tracing::debug!("Creating fs-based \"cold\" cache at {:?}", path);

        // Create the intermediate directories if they do not exist.
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tracing::debug!("Creating intermediate directories at {:?}", parent);
                std::fs::create_dir_all(parent).expect("Could not create intermediate directories");
            }
        }

        File::create(&path).expect("Could not create file system cache");
    }

    File::open(path).expect("Could not open file system cache")
}

fn get_redis_connection() -> Result<Connection> {
    let client = Client::open("redis://127.0.0.1/")?;
    let connection = client.get_connection()?;
    Ok(connection)
}

fn get_cache_dir() -> PathBuf {
    let mut path = dirs_next::cache_dir().expect("Could not find cache directory");
    path.push("rudolph");
    path.push("cache");
    path
}

// impl From<&str> for Cache
// where
//     Self: Sized,
// {
//     fn from(s: &str) -> Self {
//         let path = PathBuf::from(s);
//         let version = get_version(&path);
//         let redis = get_redis_connection();
//         let fs = get_fs_cache(&path);

//         Self { path, version, redis, fs }
//     }
// }

#[derive(Display, Getters, MutGetters, Setters, TypedBuilder)]
#[display(fmt = "rudolph_driver (v{RUDOLPH_VERSION})")]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct Driver {
    pub third_party_dir: PathBuf,
    pub reindeer:        PathBuf,
    pub cache:           Cache,
}

impl Default for Driver {
    fn default() -> Self {
        // initialize the logging subsystem
        init_logging();

        let cli = RudolphCli::parse();
        let third_party_dir = cli.third_party_dir();
        let embedded_reindeer = embedded_reindeer_path();
        let reindeer = match cli.reindeer() {
            Some(reindeer) => reindeer,
            None => &embedded_reindeer,
        };

        let cache = Cache::from(third_party_dir);
        // let cache = Cache::

        Self {
            third_party_dir: third_party_dir.to_path_buf(),
            reindeer: reindeer.to_path_buf(),
            cache,
        }
    }
}

impl Driver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn embed_reindeer(&self) -> Result<()> {
        // Check if the reindeer binary is already embedded,
        // and if so, skip the rest of the embedding process.
        if embedded_reindeer_path().exists() {
            tracing::debug!("Reindeer binary already embedded. Skipping embedding process...");
            return Ok(());
        } else {
            tracing::debug!(
                "Unable to locate preexisting embedded reindeer binary. Proceeding with embedding \
                 process..."
            );
        }

        // read the binary file
        let reindeer_bytes = include_bytes!("../reindeer");
        tracing::debug!(
            "Ingested reindeer binary. {} in total",
            format_size(reindeer_bytes.len(), BINARY) // reindeer_bytes.len() //
        );

        // write the binary file to a temporary file at the embedded path
        let mut reindeer = File::create(embedded_reindeer_path())?;
        reindeer.write_all(reindeer_bytes)?;

        // set the executable permission on the file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = reindeer.metadata()?.permissions();
            perms.set_mode(0o755);
            reindeer.set_permissions(perms)?;
        }
        Command::new("chmod").args(["+x", "reindeer"]).output()?;
        tracing::trace!("Set executable permission on reindeer binary");

        tracing::debug!("Embedded reindeer binary");

        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        // Check to see if a cached version of the third-party dependencies
        // is available and up-to-date. If so, then early return to avoid
        // unnecessary work.
        // TODO
        // if self.cache.is_up_to_date() && !self.force {
        //     return Ok(());
        // }

        // embed the reindeer binary (this is performed "lazily" and is a no-op if the
        // reindeer binary has already been embedded)
        self.embed_reindeer()?;

        // Use the embedded reindeer binary to vendor third-party Rust crates
        // into the third-party directory (e.g. `reindeer vendor --third-party-dir
        // third-party`)
        self.vendor_third_party()?;

        // Use the embedded reindeer binary to buckify third-party Rust crates
        // into the third-party directory (e.g. `reindeer buckify --third-party-dir
        // third-party`)
        self.buckify_third_party()?;

        tracing::info!(
            "Finished loading (vendoring and buckifying) third-party Rust crates into the \
             workspace."
        );
        tracing::info!(
            "This operation is incremental. Vendoring and buckifying third-party Rust crates will \
             only be performed when the dependency set inferred from the Cargo.toml file located \
             in the third-party directory has been updated."
        );
        tracing::info!(
            "To force a re-vendoring and re-buckifying of third-party Rust crates, re-run with \
             the `--force` flag.\n"
        );

        // Build all targets in the workspace (e.g. `buck2 build //...`)
        // to ensure that the newly buckified third-party crates are
        // available, and to ensure that the workspace is in a
        // consistent state.
        self.build_workspace()?;

        Ok(())
    }

    pub fn build_workspace(&self) -> Result<()> {
        tracing::info!("Building all targets in the workspace");
        let mut build = Command::new("buck2")
            .arg("build")
            .arg("//...")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let exit_status = build.wait()?;
        let (stdout, stderr) = (
            build.stdout.expect("Internal error: failed to capture stdout"),
            build.stderr.expect("Internal error: failed to capture stderr"),
        );

        let stdout = BufReader::new(stdout);
        let stderr = BufReader::new(stderr);

        let mut stdout_lines = stdout.lines().peekable();
        let mut stderr_lines = stderr.lines().peekable();

        if stdout_lines.peek().is_some() {
            tracing::info!("Captured the following stdout from `buck2 build //...`:");
            for line in stdout_lines {
                tracing::info!("{}", line?);
            }
        }

        if stderr_lines.peek().is_some() {
            tracing::info!("Captured the following stderr from `buck2 build //...`:");
            for line in stderr_lines {
                tracing::info!("{}", line?);
            }
        }

        if !exit_status.success() {
            tracing::error!("Failed to build workspace.");
            return Err(RudolphError::FailedToBuildWorkspace.into());
        }

        tracing::info!("Successfully built workspace.");
        Ok(())
    }

    pub fn vendor_third_party(&self) -> Result<()> {
        let third_party_dir = self.third_party_dir();
        let reindeer = self.reindeer();

        // if the third-party directory does not exist, return early
        if !third_party_dir.exists() {
            return Err(RudolphError::ThirdPartyDirDoesNotExist(self.third_party_dir_path()).into());
        }

        tracing::info!(
            "Vendoring Rust crates from crates.io into {}. This may take a while...",
            self.third_party_dir_path()
        );
        let mut vendor = Command::new(reindeer)
            .arg("--third-party-dir")
            .arg(third_party_dir)
            .arg("vendor")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tracing::info!(
            "Canonical command issued: {}{}{}{}{}{}",
            "`".red(),
            "reindeer".bright_yellow(),
            " --third-party-dir ".cyan().italic(),
            third_party_dir.display().to_string().bright_black().italic(),
            " vendor".yellow(),
            "`".red(),
        );

        let exit_status = vendor.wait()?;
        let (stdout, stderr) = (
            vendor.stdout.expect("Internal error: failed to capture stdout"),
            vendor.stderr.expect("Internal error: failed to capture stderr"),
        );

        let (stdout_reader, stderr_reader) = (BufReader::new(stdout), BufReader::new(stderr));

        let mut stdout_lines = stdout_reader.lines().peekable();
        if stdout_lines.peek().is_some() {
            tracing::info!("Reindeer produced the following output:");
            for line in stdout_lines {
                tracing::info!("{}", line?);
            }
        }

        let mut stderr_lines = stderr_reader.lines().peekable();
        if stderr_lines.peek().is_some() {
            tracing::warn!(
                "Reindeer encountered the following warnings/errors while vendoring third-party \
                 Rust crates:"
            );
            tracing::info!("For more information, visit the Reindeer GitHub repository:");
            tracing::info!("https://github.com/facebookincubator/reindeer");
            for line in stderr_lines {
                tracing::error!("{}", line?);
            }
            println!();
        }

        if !exit_status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Command {}{} {}{} failed with exit code {}. Please see the above output for \
                     more information.",
                    "`".bright_red(),
                    "reindeer".bright_yellow(),
                    "vendor".yellow(),
                    "`".bright_red(),
                    exit_status.code().unwrap_or(-1).to_string().bright_red()
                ),
            )
            .into());
        }

        tracing::info!("Successfully vendored third-party Rust crates.");

        Ok(())
    }

    pub fn buckify_third_party(&self) -> Result<()> {
        let third_party_dir = self.third_party_dir();
        let reindeer = self.reindeer();

        // if the third-party directory does not exist, return early
        if !third_party_dir.exists() {
            return Err(RudolphError::ThirdPartyDirDoesNotExist(self.third_party_dir_path()).into());
        }

        tracing::info!(
            "Buckifying third-party Rust crates in {}. This may take a while...",
            self.third_party_dir_path()
        );
        tracing::info!(
            "Results in a generated BUCK file encapsulating all third-party Rust crates in the \
             context of Buck2."
        );

        let mut buckify = Command::new(reindeer)
            .arg("--third-party-dir")
            .arg(third_party_dir)
            .arg("buckify")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let exit_status = buckify.wait()?;
        let (stdout, stderr) = (
            buckify.stdout.expect("Internal error: failed to capture stdout"),
            buckify.stderr.expect("Internal error: failed to capture stderr"),
        );

        let (stdout_reader, stderr_reader) = (BufReader::new(stdout), BufReader::new(stderr));

        let mut stdout_lines = stdout_reader.lines().peekable();
        if stdout_lines.peek().is_some() {
            tracing::info!("Reindeer produced the following output:");
            for line in stdout_lines {
                tracing::info!("{}", line?);
            }
            println!();
        }

        let mut stderr_lines = stderr_reader.lines();
        if stderr_lines.next().is_some() {
            tracing::warn!(
                "Reindeer encountered the following warnings/errors while buckifying third-party \
                 Rust crates:"
            );
            tracing::info!("For more information, visit the Reindeer GitHub repository:");
            tracing::info!("https://github.com/facebookincubator/reindeer");
            for line in stderr_lines {
                let line = line?;
                // split the line on the second space to get just the message suffix (i.e.
                // everything not in the prefix)
                let line = line.split_ascii_whitespace();
                let line = line.skip(2).collect::<Vec<_>>().join(" ");
                tracing::warn!(
                    "{}{}{} {}",
                    "[".bright_red(),
                    "reindeer".bright_yellow().italic(),
                    "]".bright_red(),
                    line
                );
            }
            println!();
        }

        if !exit_status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Command {}{} {}{} failed with exit code {}. Please see the above output for \
                     more information.",
                    "`".bright_red(),
                    "reindeer".bright_yellow(),
                    "buckify".yellow(),
                    "`".bright_red(),
                    exit_status.code().unwrap_or(-1).to_string().bright_red()
                ),
            )
            .into());
        }

        Ok(())
    }

    fn third_party_dir_path(&self) -> String {
        self.third_party_dir()
            .to_str()
            .expect("third-party dir path is not valid UTF-8")
            .to_string()
    }
}

fn main() -> Result<()> {
    Driver::new().run()
}

fn run_rudolph() -> Result<()> {
    // Use the embedded reindeer binary to vendor third-party Rust crates
    // into the third-party directory (e.g. `reindeer vendor --third-party-dir
    // third-party`)
    vendor_third_party()?;

    // Use the embedded reindeer binary to buckify third-party Rust crates
    // into the third-party directory (e.g. `reindeer buckify --third-party-dir
    // third-party`)
    buckify_third_party()?;

    // Build all targets in the workspace (e.g. `buck2 build //...`)
    // to ensure that the newly buckified third-party crates are
    // available, working, and up-to-date.
    build_workspace()?;

    Ok(())
}

/// Encapsulates the initialization of the context in which the program runs.
/// This includes setting up logging, embedding the reindeer binary, and
/// initializing caches.
fn init_context() -> Result<()> {
    // // parse the command line arguments
    // let args = RudolphCli::parse();

    // println!("{:#?}", args);

    // initialize the logging subsystem
    init_logging();
    // embed the reindeer binary into rudolph
    embed_reindeer()?;
    // initialize the multi-level cache
    // init_cache()?;

    Ok(())
}

fn init_cache() -> Result<()> {
    todo!()
}

fn build_workspace() -> Result<()> {
    tracing::info!("Building all targets in the workspace");
    let mut buck2_build_all = Command::new("buck2")
        .arg("build")
        .arg("//...")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // let stdout = buck2_build_all.stdout.take().unwrap();
    // let stderr = buck2_build_all.stderr.take().unwrap();

    // let stdout_reader = BufReader::new(stdout);
    // let stderr_reader = BufReader::new(stderr);

    // for line in stdout_reader.lines().chain(stderr_reader.lines()) {
    //     let line = line?;
    //     // split the line on the first space to remove the timestamp
    //     let line = line.split_once(' ').map(|x| x.1).unwrap_or(&line);
    //     tracing::info!("{}", line);
    //     // process the output line as needed
    // }

    let exit_status = buck2_build_all.wait()?;
    if !exit_status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command '{}' failed with exit code {:?}",
                "buck2 build //...",
                exit_status.code()
            ),
        )
        .into());
    }

    tracing::info!(
        "Successfully rebuilt all targets in the workspace with the latest third-party Rust \
         crates from crates.io"
    );
    Ok(())
}

fn buckify_third_party() -> Result<()> {
    tracing::info!("Buckifying third-party Rust crates. This may take a while...");
    let mut buckify = Command::new("./reindeer")
        .arg("--third-party-dir")
        .arg("third-party")
        .arg("buckify")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = buckify.stdout.take().unwrap();
    let stderr = buckify.stderr.take().unwrap();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stdout_reader.lines().chain(stderr_reader.lines()) {
        let line = line?;
        // split the line on the second space to get just the message
        let line = line.split_ascii_whitespace().nth(2).unwrap_or(&line);
        tracing::info!("{}", line);
        // process the output line as needed
    }

    tracing::info!("Buckified third-party Rust crates");
    Ok(())
}

fn vendor_third_party() -> Result<()> {
    // Check if the third-party directory exists, otherwise, return early.
    if !Path::new("third-party").exists() {
        tracing::debug!("Third-party directory does not exist");

        // get absolute path to the third-party directory
        let third_party_dir = std::env::current_dir()?.join("third-party");
        return Err(RudolphError::ThirdPartyDirDoesNotExist(
            third_party_dir.as_path().to_string_lossy().to_string(),
        )
        .into());
    }

    tracing::info!("Vendoring third-party Rust crates. This may take a while...");
    Command::new(reindeer_path())
        .arg("--third-party-dir")
        .arg("third-party")
        .arg("vendor")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?;

    tracing::info!("Vendored third-party Rust crates");
    Ok(())
}

/// The embedded reindeer binary is located in the cache directory
/// on the user's machine.
fn embedded_reindeer_path() -> PathBuf {
    let mut path = dirs_next::cache_dir().expect("Unable to find cache directory");
    path.push("reindeer");
    path
}

fn reindeer_path() -> String {
    embedded_reindeer_path().to_string_lossy().to_string()
}

/// Embeds the reindeer binary into the rudolph binary.
fn embed_reindeer() -> Result<()> {
    // Check if the reindeer binary is already embedded,
    // and if so, skip the rest of the embedding process.
    if embedded_reindeer_path().exists() {
        tracing::debug!("Reindeer binary already embedded. Skipping embedding process...");
        return Ok(());
    } else {
        tracing::debug!(
            "Unable to locate preexisting embedded reindeer binary. Proceeding with embedding \
             process..."
        );
    }

    // read the binary file
    let reindeer_bytes = include_bytes!("../reindeer");
    tracing::debug!(
        "Ingested reindeer binary. {} in total",
        format_size(reindeer_bytes.len(), BINARY) // reindeer_bytes.len() //
    );

    // write the binary file to a temporary file
    let mut reindeer = File::create("reindeer").expect("Unable to create file");
    reindeer.write_all(reindeer_bytes).expect("Unable to write data");

    // set the executable permission on the file
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = reindeer.metadata()?.permissions();
        perms.set_mode(0o755);
        reindeer.set_permissions(perms)?;
    }
    Command::new("chmod").args(["+x", "reindeer"]).output()?;
    tracing::debug!("Set executable permission on reindeer binary");

    tracing::info!("Embedded reindeer binary");

    Ok(())
}

fn init_logging() {
    // Create a subscriber with a filter that accepts all events.
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .with_max_level(tracing::Level::TRACE)
        .without_time() // turn off timestamps
        .finish();

    // Set the subscriber as the default.
    tracing::subscriber::set_global_default(subscriber).expect("failed to set subscriber");
}
