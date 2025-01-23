use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
#[clap(about = "Get opt-in status of Operator in a vault in the Symbiotic.")]
pub struct OptInStatusCommand {
    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address of the vault to opt-in."
    )]
    vault_address: Address,
}

impl OptInStatusCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
