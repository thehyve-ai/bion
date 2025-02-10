use add::AddCommand;
use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use opt_in_network::OptInNetworkCommand;
use opt_in_vault::OptInVaultCommand;
use opt_out_network::OptOutNetworkCommand;
use opt_out_vault::OptOutVaultCommand;
use register::RegisterCommand;
use remove::RemoveCommand;

mod add;
mod config;
mod consts;
mod opt_in_network;
mod opt_in_vault;
mod opt_out_network;
mod opt_out_vault;
mod register;
mod remove;
mod utils;
mod vault_parameters;

#[derive(Debug, Parser)]
#[clap(about = "Manage your operator.")]
pub struct OperatorCommand {
    #[arg(value_name = "ALIAS", help = "The saved operator alias.")]
    pub alias: String,

    #[command(subcommand)]
    pub command: OperatorSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum OperatorSubcommands {
    #[command(name = "opt-in-network")]
    OptInNetwork(OptInNetworkCommand),

    #[command(name = "opt-in_vault")]
    OptInVault(OptInVaultCommand),

    #[command(name = "opt-out-network")]
    OptOutNetwork(OptOutNetworkCommand),

    #[command(name = "opt-out-vault")]
    OptOutVault(OptOutVaultCommand),

    #[command(name = "register")]
    Register(RegisterCommand),

    // Import operator management
    #[command(name = "add")]
    Add(AddCommand),

    #[command(name = "remove")]
    Remove(RemoveCommand),
}

impl OperatorCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            OperatorSubcommands::OptInNetwork(opt_in_network) => {
                opt_in_network.with_alias(self.alias).execute(ctx).await
            }
            OperatorSubcommands::OptInVault(opt_in_vault) => {
                opt_in_vault.with_alias(self.alias).execute(ctx).await
            }
            OperatorSubcommands::OptOutNetwork(opt_out_network) => {
                opt_out_network.with_alias(self.alias).execute(ctx).await
            }
            OperatorSubcommands::OptOutVault(opt_out_vault) => {
                opt_out_vault.with_alias(self.alias).execute(ctx).await
            }
            OperatorSubcommands::Register(register) => {
                register.with_alias(self.alias).execute(ctx).await
            }
            OperatorSubcommands::Add(add) => add.with_alias(self.alias).execute(ctx).await,
            OperatorSubcommands::Remove(remove) => remove.with_alias(self.alias).execute(ctx).await,
        }
    }
}
