use alloy_primitives::{Address, B256};
use alloy_signer::Signer;
use alloy_signer_local::coins_bip39::English;
use alloy_signer_local::{LocalSigner, MnemonicBuilder, PrivateKeySigner};
use clap::Parser;
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use foundry_cli::utils;
use foundry_cli::{opts::EthereumOpts, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use itertools::Itertools;

use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::cmd::network::utils::get_network_metadata;
use crate::cmd::network::{ImportedNetworks, NetworkMetadata};
use crate::cmd::utils::{get_address_type, get_chain_id, AddressType};
use crate::common::{DirsCliArgs, SigningMethod};
use crate::symbiotic::calls::is_network;
use crate::symbiotic::consts::get_network_registry;
use crate::utils::{
    get_keystore_password, load_from_json_file, print_error_message, print_loading_until_async,
    print_success_message, read_user_confirmation, write_to_json_file, ExecuteError,
};

use super::utils::NetworkInfo;

// implementation:
// 1. bion network add <address>
// 2. check if the address is a contract
// 3. get metadata from the symbiotic info.json
// Getting network information... -> Network is <contract|EOA>
// Network is known as <info.name>. Do you want to use this name? (y/n)
// > n
// 4.a prompt name
// > y
// 4.b continue

// 5. check if the address is a registered network
// Getting network status...
// Network is <active|inactive>
// You can register the network with bion network <name> register
//

// 6. If the network is an EOA (not a contract): prompt: do you want to save the private key in a file? (y/n)
// 6.a no
// 6.a Prompt: What is your preferred signing method?
// 6.a A select menu with the options: Ledger, Keyfile, Mnemonic, Raw private key, whatever options cast has

// 6.b y -> prompt for the private key
// 6.b prompt: Do you want to create a password for the keystore file? (y/n)
// 6.b prompt: Enter a password for the keystore file

// 7
// Store the network in the config file in the datadir
// Update a network_definitions.yaml file in the datadir
// both of these you can define yourself
// network_definitions.json is an index of all the networks you have added, with structure:
// <name>: <address>
// <name>: <address>
// <name>: <address>
// ...
//
// config fiile is stored as <datadir>/<address>/config.json
// if you're storing the private key in a file, you can store it in the same directory as the config file, but as a keystore file

// config.json structure:
// {
//     "name": <name>,
//     "address": <address>,
//     "type": EOA|Multisig,
//     "signing_method": Ledger|Keyfile|Mnemonic|Raw private key|or whatever options cast has,
//     "password_enabled": true or false,
//     "date created":
//     "date updated":
//     "keystore_file": <path to the keystore file>,
//
// }

const NETWORK_DIRECTORY: &str = "networks";
const NETWORK_DEFINITIONS_FILE: &str = "network_definitions.json";
const NETWORK_CONFIG_FILE: &str = "config.json";

#[derive(Debug, Parser)]
#[clap(about = "Add a network to your bion config.")]
pub struct AddCommand {
    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    pub address: Address,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl AddCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn run(self, _ctx: CliContext) -> eyre::Result<()> {
        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;

        println!("{}{}", "Adding network: ".bright_cyan(), self.alias.bold());

        let network_definitions_path = self.dirs.data_dir(Some(chain_id))?.join(format!(
            "{}/{}",
            NETWORK_DIRECTORY, NETWORK_DEFINITIONS_FILE
        ));
        let network_config_dir = self
            .dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", NETWORK_DIRECTORY, self.address));
        let network_config_path = network_config_dir.join(NETWORK_CONFIG_FILE);

        let mut networks_map = match load_from_json_file(&network_definitions_path) {
            Ok(networks_map) => networks_map,
            Err(..) => {
                let networks_dir = self.dirs.data_dir(Some(chain_id))?.join(NETWORK_DIRECTORY);
                create_dir_all(&networks_dir).map_err(|e| {
                    eyre::eyre!(format!(
                        "Unable to create import directory: {:?}: {:?}",
                        network_definitions_path, e
                    ))
                })?;

                let networks_map = ImportedNetworks::new();
                write_to_json_file(&network_definitions_path, &networks_map, true)
                    .map_err(|e| eyre::eyre!(e))?;
                networks_map
            }
        };

        let mut network =
            match load_from_json_file::<&PathBuf, NetworkMetadata>(&network_config_path) {
                Ok(..) => {
                    print_error_message(
                        format!("\nNetwork configuration already exists: {}", self.address)
                            .as_str(),
                    );
                    return Ok(());
                }
                Err(..) => {
                    create_dir_all(&network_config_dir).map_err(|e| {
                        eyre::eyre!(format!(
                            "Unable to create network directory: {:?}: {:?}",
                            network_config_dir, e
                        ))
                    })?;

                    let network = NetworkMetadata::new(self.address, self.alias.clone());
                    write_to_json_file(&network_config_path, &network, true)
                        .map_err(|e| eyre::eyre!(e))?;
                    network
                }
            };

        let address_type = get_address_type(self.address, &provider).await?;
        network.set_address_type(address_type.clone());

        println!("\n{}{:?}", "Address type: ".bright_cyan(), address_type);

        let network_info = print_loading_until_async(
            "Fetching network metadata",
            get_network_metadata(self.address),
        )
        .await?;

        if let Some(network_alias) = self.get_network_alias(network_info)? {
            network.set_alias(network_alias);
        }

        println!(
            "\n{}{}",
            "Continuing with network alias: ".bright_cyan(),
            network.alias.bold()
        );

        // For now terminate if the alias already exists, in the future update functionality will be added
        if networks_map.contains_key(self.alias.as_str())
            || networks_map
                .values()
                .into_iter()
                .map(|a| a.to_string().to_lowercase())
                .contains(&self.address.to_string().to_lowercase())
        {
            print_error_message(
                format!("\nNetwork with alias {} already exists.", self.alias).as_str(),
            );
            return Ok(());
        }

        let is_network = print_loading_until_async(
            "Checking network status",
            is_network(self.address, network_registry, &provider),
        )
        .await?;

        if is_network {
            println!("{}", "Network is active".bright_cyan());
        } else {
            println!(
                "Network is inactive, you can register the network with bion network {} register",
                network.alias
            );
        }

        if address_type == AddressType::EOA {
            self.handle_signing_method(&mut network, &network_config_dir)?;
        }

        // store network config
        write_to_json_file(network_config_path, &network, false).map_err(|e| eyre::eyre!(e))?;

        networks_map.insert(network.alias.clone(), self.address);

        // store networks map
        write_to_json_file(network_definitions_path, &networks_map, false)
            .map_err(|e| eyre::eyre!(e))?;

        print_success_message(format!("✅ Successfully added network: {}", network.alias).as_str());

        Ok(())
    }

    fn get_network_alias(&self, network_info: Option<NetworkInfo>) -> eyre::Result<Option<String>> {
        if let Some(network) = network_info {
            println!("Network is known as: {}", network.name);
            println!(
                "\n {}",
                "Do you want to use this name instead of the provided alias? (y/n)".bright_cyan()
            );

            let confirmation: String = read_user_confirmation()?;
            return match confirmation.trim().to_lowercase().as_str() {
                "n" | "no" => Ok(None),
                _ => Ok(Some(network.name)),
            };
        }

        Ok(None)
    }

    fn handle_signing_method(
        &self,
        network: &mut NetworkMetadata,
        network_config_dir: &PathBuf,
    ) -> eyre::Result<()> {
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

                let private_key = rpassword::prompt_password_stdout("Enter private key:")?;
                let signer = foundry_wallets::utils::create_private_key_signer(&private_key)?;
                if signer.address().to_string().to_lowercase()
                    != network.address.to_string().to_lowercase()
                {
                    print_error_message("Address does not match signer!");
                    return Err(eyre::eyre!(""));
                }

                let private_key_bytes: B256 =
                    alloy_primitives::hex::FromHex::from_hex(private_key)?;

                let keystore_password = get_keystore_password()?;

                println!("✅ {}", "Keystore password setup completed".bright_green());

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &network_config_dir,
                    &mut rng,
                    private_key_bytes,
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                println!("✅ {}", "Keystore creation completed".bright_green());

                network.set_signing_method(Some(SigningMethod::Keystore));
                network.set_keystore_file(Some(network_config_dir.clone()));
                network.set_password_enabled(true);
                Ok(())
            }
            1 => {
                // Keystore
                if !store_private_key {
                    return Ok(()); // do nothing
                }

                let keypath = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter path to keystore:")
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

                let password = rpassword::prompt_password_stdout("Enter keystore password")?;

                return match PrivateKeySigner::decrypt_keystore(keypath.clone(), password) {
                    Ok(signer) => {
                        if signer.address().to_string().to_lowercase()
                            != network.address.to_string().to_lowercase()
                        {
                            print_error_message("Address does not match signer!");
                            return Err(eyre::eyre!(""));
                        }

                        network.set_signing_method(Some(SigningMethod::Keystore));
                        network.set_keystore_file(Some(keypath.into()));
                        network.set_password_enabled(true);
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
                        "Enter mnemonic phrase or path to a file that contains the phrase:",
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
                    != network.address.to_string().to_lowercase()
                {
                    print_error_message("Address does not match signer!");
                    return Err(eyre::eyre!(""));
                }

                let keystore_password = get_keystore_password()?;

                println!("✅ {}", "Keystore password setup completed".bright_green());

                let mut rng = rand::thread_rng();
                let (_, _) = LocalSigner::encrypt_keystore(
                    &network_config_dir,
                    &mut rng,
                    signer.to_bytes(),
                    keystore_password.as_ref(),
                    Some("keystore"),
                )?;

                println!("✅ {}", "Keystore creation completed".bright_green());

                network.set_signing_method(Some(SigningMethod::Keystore));
                network.set_keystore_file(Some(network_config_dir.clone()));
                network.set_password_enabled(true);

                Ok(())
            }
            3 => {
                // Ledger
                network.set_signing_method(Some(SigningMethod::Ledger));
                Ok(())
            }
            4 => {
                // Trezor
                network.set_signing_method(Some(SigningMethod::Trezor));
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
