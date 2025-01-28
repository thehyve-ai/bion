use std::str::FromStr;

use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    common::consts::TESTNET_ADDRESSES, symbiotic::calls::is_opted_in_network,
    utils::validate_cli_args,
};

const HYVE_NETWORK_ENTITY: &str = "hyve_network";
const NETWORK_OPT_IN_ENTITY: &str = "network_opt_in_service";

#[derive(Debug, Parser)]
#[clap(about = "Check the opt-in status of the Operator in the network.")]
pub struct NetworkOptInStatusCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the signer."
    )]
    address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl NetworkOptInStatusCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { address, eth } = self;

        validate_cli_args(None, &eth).await?;

        println!(
            "{}",
            "🔄 Checking if the provided address is opted in.".bright_cyan()
        );

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let hyve_network = Address::from_str(TESTNET_ADDRESSES[HYVE_NETWORK_ENTITY])?;
        let network_opt_in_service =
            alloy_primitives::Address::from_str(TESTNET_ADDRESSES[NETWORK_OPT_IN_ENTITY])?;

        let is_opted_in =
            is_opted_in_network(address, hyve_network, network_opt_in_service, &provider).await?;

        let message = if is_opted_in {
            "✅ The address is opted in.".bright_green()
        } else {
            "❌ The address is not opted in.".bright_green()
        };
        println!("{}", message);

        Ok(())
    }
}
