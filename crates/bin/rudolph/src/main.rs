use anyhow::Result;
use std::fs::File;
use std::io::{
    self,
    BufRead,
    BufReader,
    Write,
};
use std::path::Path;
use std::process::{
    Command,
    Stdio,
};
use tracing_subscriber::{
    EnvFilter,
    FmtSubscriber,
};

fn main() -> Result<()> {
    init_logging();
    embed_reindeer()?;

    vendor_third_party()?;
    buckify_third_party()?;

    build_workspace()?;

    Ok(())
}

fn build_workspace() -> Result<()> {
    let mut buck2_build_all = Command::new("buck2")
        .arg("build")
        .arg("//...")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = buck2_build_all.stdout.take().unwrap();
    let stderr = buck2_build_all.stderr.take().unwrap();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stdout_reader.lines().chain(stderr_reader.lines()) {
        let line = line?;
        // split the line on the first space to remove the timestamp
        let line = line.split_once(' ').map(|x| x.1).unwrap_or(&line);
        tracing::info!("{}", line);
        // process the output line as needed
    }

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

    tracing::info!("Successfully built all targets in the workspace");
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

fn vendor_third_party() -> Result<(), anyhow::Error> {
    tracing::info!("Vendoring third-party Rust crates. This may take a while...");
    Command::new("./reindeer")
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

fn embed_reindeer() -> Result<()> {
    // Check if the reindeer binary is already embedded,
    // and if so, skip the rest of the embedding process.
    if Path::new("reindeer").exists() {
        tracing::debug!("Reindeer binary already embedded");
        return Ok(());
    }

    // read the binary file
    let reindeer_bytes = include_bytes!("../reindeer");
    tracing::debug!(
        "Ingested reindeer binary. {} bytes in total.",
        reindeer_bytes.len() // reindeer_bytes.len().file_size(options::BINARY)
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
