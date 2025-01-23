use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
pub struct RegisterStatusCommand {
    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RegisterStatusCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
