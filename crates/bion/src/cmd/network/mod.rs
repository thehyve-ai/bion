use add::AddCommand;
use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use set_max_network_limit::SetMaxNetworkLimitCommand;

mod add;
mod set_max_network_limit;

#[derive(Debug, Parser)]
#[clap(about = "Manage your network.")]
pub struct NetworkCommand {
    #[arg(value_name = "NETWORK", help = "The saved network name.")]
    pub network: String,

    #[command(subcommand)]
    pub command: NetworkSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum NetworkSubcommands {
    #[command(name = "set-max-network-limit")]
    SetMaxNetworkLimit(SetMaxNetworkLimitCommand),

    #[command(name = "add")]
    Add(AddCommand),
}

impl NetworkCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            NetworkSubcommands::SetMaxNetworkLimit(set_max_network_limit) => {
                set_max_network_limit.execute(ctx).await
            }
            NetworkSubcommands::Add(add) => add.run(ctx).await,
        }
    }
}
