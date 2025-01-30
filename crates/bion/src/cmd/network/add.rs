use std::fs::create_dir_all;

use alloy_primitives::Address;
use chrono::format;
use clap::Parser;
use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use foundry_cli::utils;
use foundry_cli::{opts::EthereumOpts, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use itertools::Itertools;

use crate::cmd::network::utils::get_network_metadata;
use crate::cmd::network::{ImportedNetworks, NetworkMetadata};
use crate::cmd::utils::{get_address_type, get_chain_id, AddressType};
use crate::common::{DirsCliArgs, SigningMethod};
use crate::symbiotic::calls::is_network;
use crate::symbiotic::consts::get_network_registry;
use crate::utils::{
    load_from_json_file, print_error_message, print_loading_until_async, read_user_confirmation,
    write_to_json_file,
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

    pub alias: String,

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
        let networks_dir = self.dirs.data_dir().join(format!(
            "{}/{}",
            NETWORK_DIRECTORY, NETWORK_DEFINITIONS_FILE
        ));
        let network_config_dir = self.dirs.data_dir().join(format!(
            "{}/{}/{}",
            NETWORK_DIRECTORY, self.address, NETWORK_CONFIG_FILE
        ));

        let mut networks_map = match load_from_json_file(&networks_dir) {
            Ok(networks_map) => networks_map,
            Err(..) => {
                create_dir_all(&networks_dir).map_err(|e| {
                    eyre::eyre!(format!(
                        "Unable to create import directory: {:?}: {:?}",
                        networks_dir, e
                    ))
                })?;

                let networks_map = ImportedNetworks::new();
                write_to_json_file(&networks_dir, &networks_map, true)
                    .map_err(|e| eyre::eyre!(e))?;
                networks_map
            }
        };

        let mut create_new_network_config = false;
        let mut network = match load_from_json_file(&network_config_dir) {
            Ok(network) => {
                print_error_message(
                    format!("Network configuration already exists: {}", self.address).as_str(),
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

                create_new_network_config = true;
                NetworkMetadata::new(self.address, self.alias.clone())
            }
            _ => unreachable!(),
        };

        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;

        let address_type = get_address_type(self.address, &provider).await?;
        network.set_address_type(address_type.clone());

        println!("Address type: {:?}", address_type);

        let network_info = print_loading_until_async(
            "Fetching network metadata",
            get_network_metadata(self.address),
        )
        .await?;

        if let Some(network_alias) = self.get_network_alias(network_info)? {
            network.set_alias(network_alias);
        }

        println!("Continuing with network name: {}", network.alias);

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
            println!("Network is active");
        } else {
            println!(
                "Network is inactive, you can register the network with bion network {} register",
                network.alias
            );
        }

        if address_type == AddressType::EOA {
            self.handle_signing_method(&network)?;
        }

        // store network config
        write_to_json_file(network_config_dir, &network, create_new_network_config)
            .map_err(|e| eyre::eyre!(e))?;

        networks_map.insert(network.alias.clone(), self.address);

        // store networks map
        write_to_json_file(networks_dir, &networks_map, false).map_err(|e| eyre::eyre!(e))?;

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

    fn handle_signing_method(&self, mut network: &NetworkMetadata) -> eyre::Result<SigningMethod> {
        let options = vec!["Raw private key", "Ledger", "Mnemonic"];

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

        match selection {
            0 => Ok(SigningMethod::PrivateKey),
            1 => Ok(SigningMethod::Ledger),
            2 => Ok(SigningMethod::Mnemonic),
            _ => unreachable!(),
        }
    }
}
