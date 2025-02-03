use alloy_primitives::{Address, B256};
use alloy_signer::Signer;
use alloy_signer_local::{coins_bip39::English, LocalSigner, MnemonicBuilder, PrivateKeySigner};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, path::PathBuf};

use crate::{
    cmd::utils::AddressType,
    common::{DirsCliArgs, SigningMethod},
    utils::{
        get_keystore_password, print_error_message, print_success_message, read_user_confirmation,
        ExecuteError,
    },
};

use super::consts::OPERATOR_DIRECTORY;

pub type ImportedOperators = HashMap<String, Address>;

#[derive(Debug, Deserialize, Serialize)]
pub struct OperatorConfig {
    pub address: Address,
    chain_id: u64,
    pub alias: String,
    pub signing_method: Option<SigningMethod>,
    password_enabled: bool,
    date_created: i64,
    date_updated: i64,
    pub keystore_file: Option<PathBuf>,
}

impl OperatorConfig {
    pub fn new(address: Address, chain_id: u64, alias: String) -> Self {
        Self {
            address,
            chain_id,
            alias,
            signing_method: None,
            password_enabled: false,
            date_created: chrono::Utc::now().timestamp(),
            date_updated: chrono::Utc::now().timestamp(),
            keystore_file: None,
        }
    }

    pub fn set_alias(&mut self, alias: String) {
        self.alias = alias;
    }

    pub fn set_signing_method(&mut self, dirs: &DirsCliArgs) -> eyre::Result<()> {
        let data_dir = dirs.data_dir(Some(self.chain_id))?;
        let operator_config_dir = data_dir.join(format!("{}/{}", OPERATOR_DIRECTORY, self.address));

        let options = vec!["Private Key", "Keystore", "Mnemonic", "Ledger", "Trezor"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nChoose a signing method:")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|e| {
                eyre::eyre!(format!(
                    "Failed to show signing method selection menu: {}",
                    e
                ))
            })?;

        let mut store_private_key = false;
        if selection == 0 || selection == 1 || selection == 2 {
            println!(
                "\n {}",
                "Do you want to store the private key? (y/n)".bright_cyan()
            );

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
                    print_error_message("Address does not match signer!");
                    return Err(eyre::eyre!(""));
                }

                let private_key_bytes: B256 =
                    alloy_primitives::hex::FromHex::from_hex(private_key)?;

                let keystore_password = get_keystore_password()?;

                print_success_message("✅ Keystore password setup completed");

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &operator_config_dir,
                    &mut rng,
                    private_key_bytes,
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                print_success_message("✅ Keystore creation completed");

                self.signing_method = Some(SigningMethod::Keystore);
                self.keystore_file = Some(operator_config_dir.join("keystore"));
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
                        if normalized.len() == 0 {
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

                return match PrivateKeySigner::decrypt_keystore(keypath.clone(), password) {
                    Ok(signer) => {
                        if signer.address().to_string().to_lowercase()
                            != self.address.to_string().to_lowercase()
                        {
                            print_error_message("Address does not match signer!");
                            return Err(eyre::eyre!(""));
                        }

                        print_success_message("✅ Keystore successfully decrypted");

                        self.signing_method = Some(SigningMethod::Keystore);
                        self.keystore_file = Some(operator_config_dir.join("keystore"));
                        self.password_enabled = true;
                        Ok(())
                    }
                    Err(e) => Err(eyre::eyre!("Failed to decrypt keystore: {}", e)),
                };
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
                        if normalized.len() == 0 {
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

                let signer = MnemonicBuilder::<English>::default()
                    .phrase(phrase)
                    .build()?;

                if signer.address().to_string().to_lowercase()
                    != self.address.to_string().to_lowercase()
                {
                    print_error_message("Address does not match signer!");
                    return Err(eyre::eyre!(""));
                }

                let keystore_password = get_keystore_password()?;

                print_success_message("✅ Keystore password setup completed");

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &operator_config_dir,
                    &mut rng,
                    signer.to_bytes(),
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                print_success_message("✅ Keystore creation completed");

                self.signing_method = Some(SigningMethod::Keystore);
                self.keystore_file = Some(operator_config_dir.clone().join("keystore"));
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
}
