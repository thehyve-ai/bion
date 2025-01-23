use std::fs::File;
use std::path::{Path, PathBuf};

use eth2_keystore::Keystore;
use hyve_primitives::dirs::ensure_dir_exists;
use hyve_primitives::fs::write_file_via_temporary;
use lighthouse_bls::generics::GenericPublicKey;
use lighthouse_bls::PublicKey;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::{PasswordStorage, SigningDefinition};

/// The file name for the serialized `ValidatorDefinitions` struct.
pub const CONFIG_FILENAME: &str = "operator_definitions.yml";

/// The temporary file name for the serialized `ValidatorDefinitions` struct.
///
/// This is used to achieve an atomic update of the contents on disk, without truncation.
/// See: https://github.com/sigp/lighthouse/issues/2159
pub const CONFIG_TEMP_FILENAME: &str = ".operator_definitions.yml.tmp";

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorDefinition {
    pub enabled: bool,
    pub public_key: PublicKey,
    #[serde(default)]
    pub description: String,
    #[serde(flatten)]
    pub signing_definition: SigningDefinition,
}

impl OperatorDefinition {
    pub fn new_keystore_with_password<P: AsRef<Path>>(
        keystore_path: P,
        keystore_password_storage: PasswordStorage,
    ) -> Result<Self, Error> {
        let keystore_path = keystore_path.as_ref().into();
        let keystore =
            Keystore::from_json_file(&keystore_path).map_err(Error::UnableToOpenKeystore)?;
        let public_key = PublicKey::deserialize(
            keystore
                .public_key()
                .ok_or(Error::InvalidKeystorePubkey)?
                .serialize()
                .as_slice(),
        )
        .unwrap();
        let (keystore_password_path, keystore_password) = match keystore_password_storage {
            PasswordStorage::ValidatorDefinitions(password) => (None, Some(password)),
            PasswordStorage::File(path) => (Some(path), None),
            PasswordStorage::None => (None, None),
        };

        Ok(Self {
            enabled: true,
            public_key,
            description: keystore.description().unwrap_or("").to_string(),
            signing_definition: SigningDefinition::LocalKeystore {
                keystore_path,
                keystore_password_path,
                keystore_password,
            },
        })
    }
}

/// A list of `ValidatorDefinition` that serves as a serde-able configuration file which defines a
/// list of validators to be initialized by this validator client.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct OperatorDefinitions(Vec<OperatorDefinition>);

impl From<Vec<OperatorDefinition>> for OperatorDefinitions {
    fn from(vec: Vec<OperatorDefinition>) -> Self {
        Self(vec)
    }
}

impl OperatorDefinitions {
    /// Open an existing file or create a new, empty one if it does not exist.
    pub fn open_or_create<P: AsRef<Path>>(validators_dir: P) -> Result<Self, Error> {
        ensure_dir_exists(validators_dir.as_ref()).map_err(|_| {
            Error::UnableToCreateValidatorDir(PathBuf::from(validators_dir.as_ref()))
        })?;
        let config_path = validators_dir.as_ref().join(CONFIG_FILENAME);
        if !config_path.exists() {
            let this = Self::default();
            this.save(&validators_dir)?;
        }
        Self::open(validators_dir)
    }

    /// Open an existing file, returning an error if the file does not exist.
    pub fn open<P: AsRef<Path>>(validators_dir: P) -> Result<Self, Error> {
        let config_path = validators_dir.as_ref().join(CONFIG_FILENAME);
        let file = File::options()
            .write(true)
            .read(true)
            .create_new(false)
            .open(config_path)
            .map_err(Error::UnableToOpenFile)?;
        serde_yaml::from_reader(file).map_err(Error::UnableToParseFile)
    }

    /// Encodes `self` as a YAML string and atomically writes it to the `CONFIG_FILENAME` file in
    /// the `validators_dir` directory.
    ///
    /// Will create a new file if it does not exist or overwrite any existing file.
    pub fn save<P: AsRef<Path>>(&self, validators_dir: P) -> Result<(), Error> {
        let config_path = validators_dir.as_ref().join(CONFIG_FILENAME);
        let temp_path = validators_dir.as_ref().join(CONFIG_TEMP_FILENAME);
        let mut bytes = vec![];
        serde_yaml::to_writer(&mut bytes, self).map_err(Error::UnableToEncodeFile)?;

        write_file_via_temporary(&config_path, &temp_path, &bytes)
            .map_err(Error::UnableToWriteFile)?;

        Ok(())
    }

    pub fn as_slice(&self) -> &[OperatorDefinition] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, def: OperatorDefinition) {
        self.0.push(def);
    }

    pub fn remove(&mut self, public_key: &str) -> bool {
        let len = self.0.len();
        self.0
            .retain(|def| def.public_key.as_hex_string() != public_key);
        len != self.0.len()
    }
}
