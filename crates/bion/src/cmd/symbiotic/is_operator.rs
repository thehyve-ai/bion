use std::str::FromStr;

use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{common::consts::TESTNET_ADDRESSES, symbiotic::calls::get_operator_registry_status};

const OP_REGISTRY_ENTITY: &str = "op_registry";

#[derive(Debug, Parser)]
#[clap(about = "Check if the address is a Symbiotic Operator.")]
pub struct IsOperatorCommand {
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

impl IsOperatorCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { address, eth } = self;

        println!(
            "{}",
            "🔄 Checking if the provided address is registered.".bright_cyan()
        );

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let op_registry =
            alloy_primitives::Address::from_str(TESTNET_ADDRESSES[OP_REGISTRY_ENTITY])?;

        let is_opted_in = get_operator_registry_status(address, op_registry, &provider).await?;

        let message = if is_opted_in {
            "✅ The address is registered.".bright_green()
        } else {
            "❌ The address is not registered.".bright_green()
        };
        println!("{}", message);

        Ok(())
    }
}
