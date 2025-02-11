use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::utils;
use foundry_cli::{opts::EthereumOpts, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use itertools::Itertools;

use crate::cmd::network::consts::{
    NETWORK_CONFIG_FILE, NETWORK_DEFINITIONS_FILE, NETWORK_DIRECTORY,
};
use crate::cmd::network::utils::{get_or_create_network_config, get_or_create_network_definitions};
use crate::cmd::utils::{get_address_type, get_chain_id};
use crate::common::DirsCliArgs;
use crate::symbiotic::calls::is_network;
use crate::symbiotic::consts::get_network_registry;
use crate::symbiotic::network_utils::get_network_metadata;
use crate::utils::{
    print_error_message, print_loading_until_async, print_success_message, read_user_confirmation,
    write_to_json_file,
};

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

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            alias,
            dirs,
            eth,
        } = self;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;

        println!("{}{}", "Adding network: ".bright_cyan(), alias.bold());

        let network_definitions_path = dirs.data_dir(Some(chain_id))?.join(format!(
            "{}/{}",
            NETWORK_DIRECTORY, NETWORK_DEFINITIONS_FILE
        ));
        let network_config_dir = dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", NETWORK_DIRECTORY, address));
        let network_config_path = network_config_dir.join(NETWORK_CONFIG_FILE);

        let mut networks_map = get_or_create_network_definitions(chain_id, &dirs)?;

        let mut network_config =
            get_or_create_network_config(chain_id, address, alias.clone(), &dirs)?;

        let address_type = print_loading_until_async(
            "Fetching address type",
            get_address_type(address, &provider),
        )
        .await?;

        println!("\n{}{:?}", "Address type: ".bright_cyan(), address_type);

        network_config.set_address_type(address_type);

        let network_info =
            print_loading_until_async("Fetching network metadata", get_network_metadata(address))
                .await?;

        if let Some(network) = network_info {
            println!("Network is known as: {}", network.name);
            println!(
                "\n {}",
                "Do you want to use this name instead of the provided alias? (y/n)".bright_cyan()
            );

            let confirmation: String = read_user_confirmation()?;
            match confirmation.trim().to_lowercase().as_str() {
                "y" | "yes" => network_config.set_alias(network.name),
                _ => {}
            };
        }

        println!(
            "\n{}{}",
            "Continuing with network alias: ".bright_cyan(),
            network_config.alias.bold()
        );

        // For now terminate if the alias already exists, in the future update functionality will be added
        if networks_map.contains_key(alias.as_str())
            || networks_map
                .values()
                .into_iter()
                .map(|a| a.to_string().to_lowercase())
                .contains(&address.to_string().to_lowercase())
        {
            print_error_message(format!("\nNetwork with alias {} already exists.", alias).as_str());
            return Ok(());
        }

        let is_network = print_loading_until_async(
            "Checking network status",
            is_network(address, network_registry, &provider),
        )
        .await?;

        if is_network {
            println!("\n{}", "Network is active".bright_cyan());
        } else {
            println!(
                "\n{}",
                format!("Network is inactive, you can register the network with `bion network {} register`", network_config.alias)
                    .bright_cyan()
            );
        }

        network_config.set_signing_method(&dirs)?;

        write_to_json_file(network_config_path, &network_config, false)
            .map_err(|e| eyre::eyre!(e))?;

        networks_map.insert(network_config.alias.clone(), address);
        write_to_json_file(network_definitions_path, &networks_map, false)
            .map_err(|e| eyre::eyre!(e))?;

        print_success_message(
            format!("✅ Successfully added network: {}", network_config.alias).as_str(),
        );

        Ok(())
    }
}
