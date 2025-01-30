use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use set_max_network_limit::SetMaxNetworkLimitCommand;

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
}

impl NetworkCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        // find if you have the name
        let name_found = false;

        if !name_found {
            println!(
                "Network {} not found. Add the network with bion network add <name>",
                self.network
            );
            return Ok(());
        }

        match self.command {
            NetworkSubcommands::SetMaxNetworkLimit(set_max_network_limit) => {
                set_max_network_limit.execute(ctx).await
            }
        }
    }
}
