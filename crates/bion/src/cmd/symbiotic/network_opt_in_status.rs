use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::utils::get_chain_id,
    hyve::consts::get_hyve_network,
    symbiotic::{calls::is_opted_in_network, consts::get_network_opt_in_service},
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Check the opt-in status of the Operator in the network.")]
pub struct NetworkOptInStatusCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the operator."
    )]
    address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl NetworkOptInStatusCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { address, eth } = self;

        validate_cli_args(None, &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let hyve_network = get_hyve_network(chain_id)?;
        let opt_in_service = get_network_opt_in_service(chain_id)?;

        println!(
            "{}",
            "🔄 Checking if the provided operator address is opted in.".bright_cyan()
        );

        let is_opted_in =
            is_opted_in_network(address, hyve_network, opt_in_service, &provider).await?;

        let message = if is_opted_in {
            "✅ The operator is opted in.".bright_green()
        } else {
            "❌ The operator is not opted in.".red()
        };
        println!("{}", message);

        Ok(())
    }
}
