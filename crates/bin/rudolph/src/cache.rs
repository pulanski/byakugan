use anyhow::Result;
use std::path::PathBuf;

use crate::diagnostics::RudolphError;

/// Metadata about where the third-party Rust crates are located (
/// i.e. where the `third-party` directory is located)
/// and how to vendor and buckify them.
pub struct ThirdPartyDir {
    path: PathBuf,
}

impl ThirdPartyDir {
    /// The path to the third-party directory.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn new(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(RudolphError::ThirdPartyDirDoesNotExist(
                path.to_str().unwrap_or_default().to_string(),
            )
            .into());
        }
        Ok(Self { path })
    }
}
