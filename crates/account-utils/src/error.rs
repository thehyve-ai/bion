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
    UnableToWriteFile(FsError),
    /// The public key from the keystore is invalid.
    InvalidKeystorePubkey,
    /// The keystore was unable to be opened.
    UnableToOpenKeystore(eth2_keystore::Error),
    /// The validator directory could not be created.
    UnableToCreateValidatorDir(PathBuf),
    UnableToReadKeystorePassword(String),
    KeystoreWithoutPassword,
}

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("The file could not be created: {0}")]
    /// The file could not be created
    UnableToCreateFile(io::Error),
    #[error("The file could not be copied: {0}")]
    /// The file could not be copied
    UnableToCopyFile(io::Error),
    #[error("The file could not be opened: {0}")]
    /// The file could not be opened
    UnableToOpenFile(io::Error),
    #[error("The file could not be renamed: {0}")]
    /// The file could not be renamed
    UnableToRenameFile(io::Error),
    #[error("Failed to set permissions: {0}")]
    /// Failed to set permissions
    UnableToSetPermissions(io::Error),
    #[error("Failed to retrieve file metadata: {0}")]
    /// Failed to retrieve file metadata
    UnableToRetrieveMetadata(io::Error),
    #[error("Failed to write bytes to file: {0}")]
    /// Failed to write bytes to file
    UnableToWriteFile(io::Error),
    #[error("Failed to obtain file path")]
    /// Failed to obtain file path
    UnableToObtainFilePath,
    #[error("Failed to convert string to SID: {0}")]
    /// Failed to convert string to SID
    UnableToConvertSID(u32),
    #[error("Failed to retrieve ACL for file: {0}")]
    /// Failed to retrieve ACL for file
    UnableToRetrieveACL(u32),
    #[error("Failed to enumerate ACL entries: {0}")]
    /// Failed to enumerate ACL entries
    UnableToEnumerateACLEntries(u32),
    #[error("Failed to add new ACL entry: {0}")]
    /// Failed to add new ACL entry
    UnableToAddACLEntry(String),
    #[error("Failed to remove ACL entry: {0}")]
    /// Failed to remove ACL entry
    UnableToRemoveACLEntry(String),
}
