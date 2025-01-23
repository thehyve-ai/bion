use std::{io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    // /// The config file could not be opened.
    UnableToOpenFile(io::Error),
    // /// The config file could not be parsed as YAML.
    UnableToParseFile(serde_yaml::Error),
    // /// There was an error whilst performing the recursive keystore search function.
    // UnableToSearchForKeystores(io::Error),
    // /// The config file could not be serialized as YAML.
    UnableToEncodeFile(serde_yaml::Error),
    // /// The config file or temp file could not be written to the filesystem.
    UnableToWriteFile(hyve_primitives::fs::FsError),
    /// The public key from the keystore is invalid.
    InvalidKeystorePubkey,
    /// The keystore was unable to be opened.
    UnableToOpenKeystore(eth2_keystore::Error),
    /// The validator directory could not be created.
    UnableToCreateValidatorDir(PathBuf),
    UnableToReadKeystorePassword(String),
    KeystoreWithoutPassword,
}
