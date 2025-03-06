use clap::Parser;
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
pub struct ListOperatorsCommand {
    #[arg(skip)]
    alias: String,
}

impl ListOperatorsCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
