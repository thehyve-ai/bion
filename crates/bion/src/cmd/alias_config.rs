use alloy_primitives::{Address, B256};
use alloy_signer::Signer;
use alloy_signer_local::{coins_bip39::English, LocalSigner, MnemonicBuilder, PrivateKeySigner};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use foundry_common::provider::RetryProvider;
use itertools::Itertools;
use safe_multisig::calls::get_owners;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, path::PathBuf};

use crate::{
    cmd::utils::AddressType,
    common::{DirsCliArgs, SigningMethod},
    utils::{get_keystore_password, print_success_message, read_user_confirmation, ExecuteError},
};

use super::consts::ALIAS_DIRECTORY;

pub type ImportedAliases = HashMap<String, Address>;

#[derive(Debug, Deserialize, Serialize)]
pub struct AliasConfig {
    pub address: Address,
    chain_id: u64,
    pub alias: String,
    address_type: AddressType, // default value that will be overwritten
    pub signing_method: Option<SigningMethod>,
    pub owner_signing_method: Option<SigningMethod>,
    password_enabled: bool,
    date_created: i64,
    date_updated: i64,
    pub keystore_file: Option<PathBuf>,
}

impl AliasConfig {
    pub fn new(address: Address, chain_id: u64, alias: String) -> Self {
        Self {
            address,
            chain_id,
            alias,
            address_type: AddressType::EOA,
            signing_method: None,
            owner_signing_method: None,
            password_enabled: false,
            date_created: chrono::Utc::now().timestamp(),
            date_updated: chrono::Utc::now().timestamp(),
            keystore_file: None,
        }
    }

    pub fn set_address_type(&mut self, address_type: AddressType) {
        self.address_type = address_type;
    }

    pub async fn set_signing_method(
        &mut self,
        dirs: &DirsCliArgs,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        if self.address_type == AddressType::Contract {
            return self.handle_multisig_signing_method(dirs, provider).await;
        }

        let data_dir = dirs.data_dir(Some(self.chain_id))?;
        let alias_config_dir = data_dir.join(format!("{}/{}", ALIAS_DIRECTORY, self.address));

        let options = vec!["Private Key", "Keystore", "Mnemonic", "Ledger", "Trezor"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nChoose a signing method:")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|e| {
                eyre::eyre!(format!("Failed to show signing method selection menu: {}", e))
            })?;

        let mut store_private_key = false;
        if selection == 0 || selection == 1 || selection == 2 {
            println!("\n {}", "Do you want to store the private key? (y/n)".bright_cyan());

            let confirmation: String = read_user_confirmation()?;
            if confirmation.trim().to_lowercase().as_str() == "y"
                || confirmation.trim().to_lowercase().as_str() == "yes"
            {
                store_private_key = true;
            }
        }

        match selection {
            0 => {
                // Private key
                if !store_private_key {
                    return Ok(()); // do nothing
                }

                let private_key = rpassword::prompt_password_stdout("\nEnter private key:")?;
                let signer = foundry_wallets::utils::create_private_key_signer(&private_key)?;
                if signer.address().to_string().to_lowercase()
                    != self.address.to_string().to_lowercase()
                {
                    eyre::bail!("Address does not match signer!");
                }

                let private_key_bytes: B256 =
                    alloy_primitives::hex::FromHex::from_hex(private_key)?;

                let keystore_password = get_keystore_password()?;

                print_success_message("✅ Keystore password setup completed");

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &alias_config_dir,
                    &mut rng,
                    private_key_bytes,
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                print_success_message("✅ Keystore creation completed");

                self.signing_method = Some(SigningMethod::Keystore);
                self.keystore_file = Some(alias_config_dir.join("keystore"));
                self.password_enabled = true;
                Ok(())
            }
            1 => {
                // Keystore
                if !store_private_key {
                    return Ok(()); // do nothing
                }

                let keypath = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("\nEnter path to keystore:")
                    .validate_with(|input: &String| -> std::result::Result<(), &str> {
                        let normalized = input.trim().to_lowercase();
                        if normalized.is_empty() {
                            Err("Keystore path must not be empty.")
                        } else {
                            Ok(())
                        }
                    })
                    .interact()
                    .map_err(|e: dialoguer::Error| match e {
                        dialoguer::Error::IO(e) => match e.kind() {
                            std::io::ErrorKind::Interrupted => ExecuteError::UserCancelled,
                            _ => ExecuteError::Other(e.into()),
                        },
                    })?;

                let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;

                match PrivateKeySigner::decrypt_keystore(keypath.clone(), password) {
                    Ok(signer) => {
                        if signer.address().to_string().to_lowercase()
                            != self.address.to_string().to_lowercase()
                        {
                            eyre::bail!("Address does not match signer!");
                        }

                        print_success_message("✅ Keystore successfully decrypted");

                        self.signing_method = Some(SigningMethod::Keystore);
                        self.keystore_file = Some(alias_config_dir.join("keystore"));
                        self.password_enabled = true;
                        Ok(())
                    }
                    Err(e) => Err(eyre::eyre!("Failed to decrypt keystore: {}", e)),
                }
            }
            2 => {
                // Mnemonic
                if !store_private_key {
                    return Ok(()); // do nothing
                }

                let phrase = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "\nEnter mnemonic phrase or path to a file that contains the phrase:",
                    )
                    .validate_with(|input: &String| -> std::result::Result<(), &str> {
                        let normalized = input.trim().to_lowercase();
                        if normalized.is_empty() {
                            Err("Mnemonic phrase or path cannot be empty.")
                        } else {
                            Ok(())
                        }
                    })
                    .interact()
                    .map_err(|e: dialoguer::Error| match e {
                        dialoguer::Error::IO(e) => match e.kind() {
                            std::io::ErrorKind::Interrupted => ExecuteError::UserCancelled,
                            _ => ExecuteError::Other(e.into()),
                        },
                    })?;

                let signer = MnemonicBuilder::<English>::default().phrase(phrase).build()?;

                if signer.address().to_string().to_lowercase()
                    != self.address.to_string().to_lowercase()
                {
                    eyre::bail!("Address does not match signer!");
                }

                let keystore_password = get_keystore_password()?;

                print_success_message("✅ Keystore password setup completed");

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &alias_config_dir,
                    &mut rng,
                    signer.to_bytes(),
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                print_success_message("✅ Keystore creation completed");

                self.signing_method = Some(SigningMethod::Keystore);
                self.keystore_file = Some(alias_config_dir.clone().join("keystore"));
                self.password_enabled = true;

                Ok(())
            }
            3 => {
                // Ledger
                self.signing_method = Some(SigningMethod::Ledger);
                Ok(())
            }
            4 => {
                // Trezor
                self.signing_method = Some(SigningMethod::Trezor);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub fn set_password_enabled(&mut self, password_enabled: bool) {
        self.password_enabled = password_enabled;
    }

    pub fn set_keystore_file(&mut self, keystore_file: Option<PathBuf>) {
        self.keystore_file = keystore_file;
    }

    async fn handle_multisig_signing_method(
        &mut self,
        dirs: &DirsCliArgs,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        let Ok(owners) = get_owners(self.address, provider).await else {
            eyre::bail!("Please verify that the provided address is a multisig contract.");
        };

        let data_dir = dirs.data_dir(Some(self.chain_id))?;
        let alias_config_dir = data_dir.join(format!("{}/{}", ALIAS_DIRECTORY, self.address));

        let options = vec!["Private Key", "Keystore", "Mnemonic", "Ledger", "Trezor"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nChoose how you prefer to import an owner account:")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|e| {
                eyre::eyre!(format!("Failed to show owner import selection menu: {}", e))
            })?;

        match selection {
            0 => {
                // Private key
                let private_key = rpassword::prompt_password_stdout("\nEnter private key:")?;
                let signer = foundry_wallets::utils::create_private_key_signer(&private_key)?;
                if !owners.iter().contains(&signer.address()) {
                    eyre::bail!("Not an owner")
                }

                let private_key_bytes: B256 =
                    alloy_primitives::hex::FromHex::from_hex(private_key)?;

                let keystore_password = get_keystore_password()?;

                print_success_message("✅ Keystore password setup completed");

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &alias_config_dir,
                    &mut rng,
                    private_key_bytes,
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                print_success_message("✅ Keystore creation completed");

                self.owner_signing_method = Some(SigningMethod::Keystore);
                self.keystore_file = Some(alias_config_dir.join("keystore"));
                self.password_enabled = true;
            }
            1 => {
                // Keystore
                let keypath = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("\nEnter path to keystore:")
                    .validate_with(|input: &String| -> std::result::Result<(), &str> {
                        let normalized = input.trim().to_lowercase();
                        if normalized.is_empty() {
                            Err("Keystore path must not be empty.")
                        } else {
                            Ok(())
                        }
                    })
                    .interact()
                    .map_err(|e: dialoguer::Error| match e {
                        dialoguer::Error::IO(e) => match e.kind() {
                            std::io::ErrorKind::Interrupted => ExecuteError::UserCancelled,
                            _ => ExecuteError::Other(e.into()),
                        },
                    })?;

                let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;

                match PrivateKeySigner::decrypt_keystore(keypath.clone(), password) {
                    Ok(signer) => {
                        if !owners.iter().contains(&signer.address()) {
                            eyre::bail!("Not an owner");
                        }

                        print_success_message("✅ Keystore successfully decrypted");

                        self.owner_signing_method = Some(SigningMethod::Keystore);
                        self.keystore_file = Some(alias_config_dir.join("keystore"));
                        self.password_enabled = true;
                    }
                    Err(e) => eyre::bail!("Failed to decrypt keystore: {}", e),
                };
            }
            2 => {
                // Mnemonic
                let phrase = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(
                        "\nEnter mnemonic phrase or path to a file that contains the phrase:",
                    )
                    .validate_with(|input: &String| -> std::result::Result<(), &str> {
                        let normalized = input.trim().to_lowercase();
                        if normalized.is_empty() {
                            Err("Mnemonic phrase or path cannot be empty.")
                        } else {
                            Ok(())
                        }
                    })
                    .interact()
                    .map_err(|e: dialoguer::Error| match e {
                        dialoguer::Error::IO(e) => match e.kind() {
                            std::io::ErrorKind::Interrupted => ExecuteError::UserCancelled,
                            _ => ExecuteError::Other(e.into()),
                        },
                    })?;

                let signer = MnemonicBuilder::<English>::default().phrase(phrase).build()?;

                if !owners.iter().contains(&signer.address()) {
                    eyre::bail!("Not an owner");
                }

                let keystore_password = get_keystore_password()?;

                print_success_message("✅ Keystore password setup completed");

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &alias_config_dir,
                    &mut rng,
                    signer.to_bytes(),
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                print_success_message("✅ Keystore creation completed");

                self.owner_signing_method = Some(SigningMethod::Keystore);
                self.keystore_file = Some(alias_config_dir.clone().join("keystore"));
                self.password_enabled = true;
            }
            3 => {
                // Ledger
                self.owner_signing_method = Some(SigningMethod::Ledger);
            }
            4 => {
                // Trezor
                self.owner_signing_method = Some(SigningMethod::Trezor);
            }
            _ => unreachable!(),
        }

        self.signing_method = Some(SigningMethod::MultiSig);
        Ok(())
    }
}
