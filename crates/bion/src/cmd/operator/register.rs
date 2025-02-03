use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::EthereumOpts;
use hyve_cli_runner::CliContext;

use crate::common::DirsCliArgs;

#[derive(Debug, Parser)]
pub struct RegisterCommand {
    #[arg(value_name = "ADDRESS", help = "The address to register.")]
    pub address: Address,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RegisterCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
