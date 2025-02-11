use alloy_primitives::{aliases::U96, Address};
use clap::Parser;
use foundry_cli::opts::EthereumOpts;
use hyve_cli_runner::CliContext;

use crate::common::DirsCliArgs;

#[derive(Debug, Parser)]
pub struct VaultParametersCommand {
    #[arg(value_name = "ADDRESS", help = "Address of the vault.")]
    vault: Address,

    #[arg(value_name = "NETWORK", help = "The network address.")]
    network: Address,

    #[arg(value_name = "SUBNETWORK", help = "The subnetwork identifier.")]
    subnetwork: U96,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl VaultParametersCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
