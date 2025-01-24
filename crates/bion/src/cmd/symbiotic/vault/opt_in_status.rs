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
    common::consts::TESTNET_ADDRESSES, symbiotic::calls::get_operator_vault_opt_in_status,
};

const VAULT_OPT_IN_ENTITY: &str = "vault_opt_in_service";

#[derive(Debug, Parser)]
#[clap(about = "Get opt-in status of Operator in a vault in the Symbiotic.")]
pub struct OptInStatusCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the signer."
    )]
    address: Address,

    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address of the vault."
    )]
    vault_address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl OptInStatusCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            vault_address,
            eth,
        } = self;

        println!(
            "{}",
            "🔄 Checking if the provided address is opted in.".bright_cyan()
        );

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let vault_opt_in_service =
            alloy_primitives::Address::from_str(TESTNET_ADDRESSES[VAULT_OPT_IN_ENTITY])?;

        let is_opted_in = get_operator_vault_opt_in_status(
            address,
            vault_address,
            vault_opt_in_service,
            &provider,
        )
        .await?;

        let message = if is_opted_in {
            "✅ The address is opted in.".bright_green()
        } else {
            "❌ The address is not opted in.".bright_green()
        };
        println!("{}", message);

        Ok(())
    }
}
