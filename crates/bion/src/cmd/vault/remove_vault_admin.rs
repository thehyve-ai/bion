use clap::Parser;
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
pub struct RemoveVaultAdminCommand {}

impl RemoveVaultAdminCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        Ok(())
    }
}
