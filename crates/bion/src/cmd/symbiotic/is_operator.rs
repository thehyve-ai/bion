use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    symbiotic::{calls::is_operator, consts::get_operator_registry},
    utils::{try_get_chain, validate_cli_args},
};

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
            "🔄 Checking if the provided operator is registered.".bright_cyan()
        );

        validate_cli_args(None, &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = {
            let cast = cast::Cast::new(&provider);
            cast.chain_id().await?
        };
        let op_registry = get_operator_registry(chain_id)?;

        let is_opted_in = is_operator(address, op_registry, &provider).await?;

        let message = if is_opted_in {
            "✅ The operator is registered.".bright_green()
        } else {
            "❌ The operator is not registered.".bright_green()
        };
        println!("{}", message);

        Ok(())
    }
}
