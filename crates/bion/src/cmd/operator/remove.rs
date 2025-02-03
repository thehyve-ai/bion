use clap::Parser;
use foundry_cli::opts::EthereumOpts;
use hyve_cli_runner::CliContext;

use crate::common::DirsCliArgs;

#[derive(Debug, Parser)]
pub struct RemoveCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RemoveCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
