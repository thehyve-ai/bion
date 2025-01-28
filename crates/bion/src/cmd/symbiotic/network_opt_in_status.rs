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
    common::consts::TESTNET_ADDRESSES,
    symbiotic::{calls::is_opted_in_network, consts::get_network_opt_in_service},
    utils::{try_get_chain, validate_cli_args},
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

        println!(
            "{}",
            "🔄 Checking if the provided operator address is opted in.".bright_cyan()
        );

        validate_cli_args(None, &eth).await?;

        let chain = try_get_chain(&eth.etherscan)?;
        let hyve_network = Address::from_str(TESTNET_ADDRESSES[HYVE_NETWORK_ENTITY])?;
        let opt_in_service = get_network_opt_in_service(chain)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let is_opted_in =
            is_opted_in_network(address, hyve_network, opt_in_service, &provider).await?;

        let message = if is_opted_in {
            "✅ The operator is opted in.".bright_green()
        } else {
            "❌ The operator is not opted in.".bright_green()
        };
        println!("{}", message);

        Ok(())
    }
}
