use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
#[clap(about = "Check the opt-in status of the Operator in the network.")]
pub struct OptInStatusCommand {
    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl OptInStatusCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
