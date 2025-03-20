use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;

use std::time::Instant;

use crate::{
    cmd::utils::get_chain_id,
    symbiotic::{
        consts::get_network_registry,
        network_utils::{validate_network_symbiotic_status, NetworkData, NetworkDataTableBuilder},
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for a Symbiotic network.")]
pub struct GetNetworkCommand {
    #[arg(value_name = "NETWORK", help = "The address of the network.")]
    network: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl GetNetworkCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { network, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;

        validate_network_symbiotic_status(network, network_registry, &provider).await?;

        let t1 = Instant::now();
        let network = print_loading_until_async(
            "Loading network",
            NetworkData::load(network, chain_id, &provider),
        )
        .await?;

        println!(
            "{}\n",
            format!("Loaded network in {}ms", t1.elapsed().as_millis()).bright_green()
        );

        let table_builder = NetworkDataTableBuilder::from_network_data(network);
        let table_builder = table_builder.with_all();
        let table = table_builder.build();
        table.printstd();

        Ok(())
    }
}
