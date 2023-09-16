use codespan_reporting::term;
use codespan_reporting::term::termcolor::{
    ColorChoice,
    StandardStream,
};
use eyre::Result;
use std::collections::HashMap;
use std::env;
use std::env::temp_dir;
use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
// use tempfile::TempDir;
use thiserror::Error;
use ulid::Ulid;

pub use codespan_reporting::diagnostic::Severity;

pub use codespan::FileId;

pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type Label = codespan_reporting::diagnostic::Label<FileId>;
pub type Files = codespan::Files<String>;

// Add a new error variant for toolchain management
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Sandbox not found: {0}")]
    SandboxNotFound(Ulid),

    #[error("Failed to create sandbox: {0}")]
    SandboxCreationError(String),

    #[error("Failed to run command in sandbox: {0}")]
    CommandExecutionError(String),

    #[error("Failed to manage Rust toolchain: {0}")]
    ToolchainError(String),
}

// New Toolchain struct for managing Rust toolchain versions
pub struct Toolchain {
    version: String,
    path:    PathBuf,
}

impl Toolchain {
    pub fn new(version: &str) -> Result<Self, SandboxError> {
        // let temp_dir = TempDir::new()?;
        let temp_dir = temp_dir();
        let toolchain_path = temp_dir.as_path().join("rust-toolchain");
        Self::download_toolchain(version, &toolchain_path)?;
        Ok(Self { version: version.to_string(), path: toolchain_path })
    }

    fn download_toolchain(version: &str, path: &Path) -> Result<(), SandboxError> {
        // Download and install the Rust toolchain using `rustup-init` or precompiled
        // binaries For the demo, we assume the toolchain is installed at the
        // given path
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub struct Sandbox {
    id:          Ulid,
    root:        PathBuf,
    cargo_home:  PathBuf,
    rustup_home: PathBuf,
    toolchain:   Toolchain,
}

impl Sandbox {
    pub fn new(root: PathBuf, toolchain: Toolchain) -> Self {
        let id = Ulid::new();
        let cargo_home = root.join("cargo_home");
        let rustup_home = root.join("rustup_home");
        fs::create_dir_all(&cargo_home).unwrap();
        fs::create_dir_all(&rustup_home).unwrap();

        Self { id, root, cargo_home, rustup_home, toolchain }
    }

    pub fn run_command(&self, command: &str) -> Result<(), SandboxError> {
        // Set the appropriate environment variables and toolchain paths before running
        // the command
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .env("CARGO_HOME", &self.cargo_home)
            .env("RUSTUP_HOME", &self.rustup_home)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.toolchain.path().join("bin").display(),
                    env::var("PATH").unwrap()
                ),
            )
            .current_dir(&self.root)
            .output()
            .map_err(SandboxError::IoError)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SandboxError::CommandExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub fn run_cargo_command(&self, command: &str) -> Result<(), SandboxError> {
        self.run_command(&format!("cargo {}", command))
    }

    pub fn run_cargo_build(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("build")
    }

    pub fn run_cargo_test(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("test")
    }

    pub fn run_cargo_run(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("run")
    }

    pub fn run_cargo_check(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("check")
    }

    pub fn run_cargo_clippy(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("clippy")
    }

    pub fn run_cargo_doc(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("doc")
    }

    pub fn run_cargo_bench(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("bench")
    }

    pub fn run_cargo_fmt(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("fmt")
    }

    pub fn run_cargo_clean(&self) -> Result<(), SandboxError> {
        self.run_cargo_command("clean")
    }
}

pub struct SandboxManager {
    sandboxes: HashMap<Ulid, Sandbox>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self { sandboxes: HashMap::new() }
    }

    pub fn create_sandbox(
        &mut self,
        root: PathBuf,
        toolchain: Toolchain,
    ) -> Result<Ulid, SandboxError> {
        let sandbox = Sandbox::new(root, toolchain);
        let id = sandbox.id;
        self.sandboxes.insert(id, sandbox);
        Ok(id)
    }

    pub fn get_sandbox(&self, id: Ulid) -> Result<&Sandbox, SandboxError> {
        self.sandboxes.get(&id).ok_or(SandboxError::SandboxNotFound(id))
    }

    pub fn run_command(&self, id: Ulid, command: &str) -> Result<(), SandboxError> {
        let sandbox = self.get_sandbox(id)?;
        sandbox.run_command(command)
    }
}

fn main() -> Result<(), eyre::Report> {
    let mut manager = SandboxManager::new();
    let root = PathBuf::from("/tmp/sandbox");
    let toolchain = Toolchain::new("stable")?;
    let sandbox_id = manager.create_sandbox(root, toolchain)?;
    let mut diagnostic = Diagnostic::error();

    if let Err(e) = manager.run_command(sandbox_id, "cargo --version") {
        diagnostic = Diagnostic::error()
            .with_message("Failed to run command in sandbox")
            .with_code("sandbox::command_failed")
            .with_notes(vec![unindent::unindent(
                "For more information, try running the command with `RUST_BACKTRACE=1`",
            )])

        // return Err(eyre::eyre!(e));
    }

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();
    term::emit::<Files>(&mut writer.lock(), &config, &Default::default(), &diagnostic)?;

    Ok(())
}
