use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
#[clap(about = "Set a max network limit on specific vault for your network.")]
pub struct SetMaxNetworkLimitCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    pub vault: Address,

    #[arg(value_name = "SUBNET", help = "The subnet to set the limit for.")]
    pub subnet: usize,

    #[arg(value_name = "LIMIT", help = "The limit to set.")]
    pub limit: usize,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

impl SetMaxNetworkLimitCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
