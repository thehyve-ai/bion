use std::fs::remove_dir_all;

use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::{
        network::{
            consts::{NETWORK_DEFINITIONS_FILE, NETWORK_DIRECTORY},
            utils::get_or_create_network_definitions,
        },
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    utils::{print_error_message, print_success_message, write_to_json_file},
};

#[derive(Debug, Parser)]
pub struct RemoveCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RemoveCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { alias, dirs, eth } = self;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;

        println!(
            "{}",
            "🔄 Checking if a network with the provided alias is imported.".bright_cyan()
        );

        let mut networks_map = get_or_create_network_definitions(chain_id, &dirs)?;
        if let Some((_, network_address)) = networks_map.get_key_value(&alias) {
            let data_dir = dirs.data_dir(Some(chain_id))?;
            let networks_dir = data_dir.join(NETWORK_DIRECTORY);
            let network_config_dir = networks_dir.join(network_address.to_string());
            let network_definitions_path = networks_dir.join(NETWORK_DEFINITIONS_FILE);

            // remove the network config folder
            remove_dir_all(&network_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to remove network config directory: {:?}: {:?}",
                    network_config_dir, e
                ))
            })?;

            networks_map.remove(&alias);
            write_to_json_file(network_definitions_path, &networks_map, false)
                .map_err(|e| eyre::eyre!(e))?;

            print_success_message(format!("✅ Successfully removed network: {}", alias).as_str());
        } else {
            print_error_message("Network with the provided alias is not imported.");
        }

        Ok(())
    }
}
