// Contains code from Lighthouse

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

/// Names for the default directories.
pub const DEFAULT_ROOT_DIR: &str = ".bion";
pub const DEFAULT_SECRET_DIR: &str = "secrets";
pub const DEFAULT_WALLET_DIR: &str = "wallets";
pub const DEFAULT_OPERATOR_DIR: &str = "operators";
pub const DEFAULT_NETWORK_DIR: &str = "network";
pub const DEFAULT_KEYSTORE_DIR: &str = "keystores";
pub const DEFAULT_KEYSTORE_FILENAME: &str = "voting-keystore.json";

/// Checks if a directory exists in the given path and creates a directory if it does not exist.
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();

    if !path.exists() {
        create_dir_all(path).map_err(|e| format!("Unable to create {:?}: {:?}", path, e))?;
    }

    Ok(())
}

/// Get the default hyve directory.
pub fn bion_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(DEFAULT_ROOT_DIR))
}

pub fn keystores_dir() -> Option<PathBuf> {
    bion_dir().map(|root| root.join(DEFAULT_KEYSTORE_DIR))
}
