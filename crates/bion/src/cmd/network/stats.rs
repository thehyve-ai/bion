use clap::Parser;
use hyve_cli_runner::CliContext;

#[derive(Debug, Parser)]
#[clap(about = "Display the stats of Hyve.")]
pub struct StatsCommand {}

impl StatsCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        todo!()
    }
}
