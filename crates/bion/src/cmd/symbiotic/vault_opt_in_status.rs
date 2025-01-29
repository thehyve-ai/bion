use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    symbiotic::{
        calls::{is_opted_in_vault, is_vault},
        consts::{get_vault_factory, get_vault_opt_in_service},
    },
    utils::{try_get_chain, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get opt-in status of Operator in a vault in the Symbiotic.")]
pub struct VaultOptInStatusCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the operator."
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

impl VaultOptInStatusCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            vault_address,
            eth,
        } = self;

        validate_cli_args(None, &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = {
            let cast = cast::Cast::new(&provider);
            cast.chain_id().await?
        };

        let opt_in_service = get_vault_opt_in_service(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        let is_vault = is_vault(vault_address, vault_factory, &provider).await?;
        if !is_vault {
            return Err(eyre::eyre!("Address is not a vault."));
        }

        println!(
            "{}",
            "🔄 Checking if the operator is opted in.".bright_cyan()
        );

        let is_opted_in =
            is_opted_in_vault(address, vault_address, opt_in_service, &provider).await?;

        let message = if is_opted_in {
            "✅ The operator is opted in.".bright_green()
        } else {
            "❌ The operator is not opted in.".bright_green()
        };
        println!("{}", message);

        Ok(())
    }
}
