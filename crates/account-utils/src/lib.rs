use error::FsError;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

use self::error::Error;
use self::helpers::read_password_string;

pub mod error;
pub mod helpers;
pub mod mnemonic;
pub mod operator_definitions;

pub use bip39;

pub use self::operator_definitions::*;

/// Defines how a password for a validator keystore will be persisted.
pub enum PasswordStorage {
    /// Store the password in the `validator_definitions.yml` file.
    ValidatorDefinitions(ZeroizeString),
    /// Store the password in a separate, dedicated file (likely in the "secrets" directory).
    File(PathBuf),
    /// Don't store the password at all.
    None,
}

/// Provides a new-type wrapper around `String` that is zeroized on `Drop`.
///
/// Useful for ensuring that password memory is zeroed-out on drop.
#[derive(Clone, PartialEq, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
#[serde(transparent)]
pub struct ZeroizeString(String);

impl From<String> for ZeroizeString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl ZeroizeString {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Remove any number of newline or carriage returns from the end of a vector of bytes.
    pub fn without_newlines(&self) -> ZeroizeString {
        let stripped_string = self.0.trim_end_matches(['\r', '\n']).into();
        Self(stripped_string)
    }
}

impl AsRef<[u8]> for ZeroizeString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SigningDefinition {
    /// A validator that is defined by an EIP-2335 keystore on the local filesystem.
    #[serde(rename = "local_keystore")]
    LocalKeystore {
        keystore_path: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        keystore_password_path: Option<PathBuf>,
        #[serde(skip_serializing_if = "Option::is_none")]
        keystore_password: Option<ZeroizeString>,
    },
    #[serde(rename = "testing_key")]
    TestingKey { private_key: [u8; 32] },
}

impl SigningDefinition {
    pub fn keystore_password(&self) -> Result<Option<ZeroizeString>, Error> {
        match self {
            SigningDefinition::LocalKeystore { keystore_password: Some(password), .. } => {
                Ok(Some(password.clone()))
            }
            SigningDefinition::LocalKeystore { keystore_password_path: Some(path), .. } => {
                read_password_string(path)
                    .map(Into::into)
                    .map(Option::Some)
                    .map_err(Error::UnableToReadKeystorePassword)
            }
            SigningDefinition::LocalKeystore { .. } => Err(Error::KeystoreWithoutPassword),
            SigningDefinition::TestingKey { .. } => Ok(None),
        }
    }

    pub fn keystore_path(&self) -> Option<&Path> {
        match self {
            SigningDefinition::LocalKeystore { keystore_path, .. } => Some(keystore_path),
            SigningDefinition::TestingKey { .. } => None,
        }
    }
}

/// Write a file atomically by using a temporary file as an intermediate.
///
/// Care is taken to preserve the permissions of the file at `file_path` being written.
///
/// If no file exists at `file_path` one will be created with restricted 0o600-equivalent
/// permissions.
pub fn write_file_via_temporary(
    file_path: &Path,
    temp_path: &Path,
    bytes: &[u8],
) -> Result<(), FsError> {
    // If the file already exists, preserve its permissions by copying it.
    // Otherwise, create a new file with restricted permissions.
    if file_path.exists() {
        fs::copy(file_path, temp_path).map_err(FsError::UnableToCopyFile)?;
        fs::write(temp_path, bytes).map_err(FsError::UnableToWriteFile)?;
    } else {
        create_with_600_perms(temp_path, bytes)?;
    }

    // With the temporary file created, perform an atomic rename.
    fs::rename(temp_path, file_path).map_err(FsError::UnableToRenameFile)?;

    Ok(())
}

/// Creates a file with `600 (-rw-------)` permissions and writes the specified bytes to file.
pub fn create_with_600_perms<P: AsRef<Path>>(path: P, bytes: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let mut file = File::create(path).map_err(FsError::UnableToCreateFile)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = file.metadata().map_err(FsError::UnableToRetrieveMetadata)?.permissions();
        perm.set_mode(0o600);
        file.set_permissions(perm).map_err(FsError::UnableToSetPermissions)?;
    }

    file.write_all(bytes).map_err(FsError::UnableToWriteFile)?;
    #[cfg(windows)]
    {
        restrict_file_permissions(path)?;
    }

    Ok(())
}
