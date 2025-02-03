use add::AddCommand;
use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use register::RegisterCommand;
use remove::RemoveCommand;
use set_max_network_limit::SetMaxNetworkLimitCommand;

mod add;
pub(crate) mod config;
pub(crate) mod consts;
mod register;
mod remove;
mod set_max_network_limit;
mod utils;

#[derive(Debug, Parser)]
#[clap(about = "Manage your network.")]
pub struct NetworkCommand {
    #[arg(value_name = "ALIAS", help = "The saved network alias.")]
    pub alias: String,

    #[command(subcommand)]
    pub command: NetworkSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum NetworkSubcommands {
    #[command(name = "register")]
    Register(RegisterCommand),

    #[command(name = "set-max-network-limit")]
    SetMaxNetworkLimit(SetMaxNetworkLimitCommand),

    // Import network management
    #[command(name = "add")]
    Add(AddCommand),

    #[command(name = "remove")]
    Remove(RemoveCommand),
}

impl NetworkCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            NetworkSubcommands::Register(register) => {
                register.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::SetMaxNetworkLimit(set_max_network_limit) => {
                set_max_network_limit.execute(ctx).await
            }
            NetworkSubcommands::Add(add) => add.with_alias(self.alias).execute(ctx).await,
            NetworkSubcommands::Remove(remove) => remove.with_alias(self.alias).execute(ctx).await,
        }
    }
}
