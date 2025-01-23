use async_trait::async_trait;
use clap::Parser;
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
#[clap(about = "Exports all BLS keys and password files.")]
pub struct ExportCommand {}

impl ExportCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        todo!()
    }
}
