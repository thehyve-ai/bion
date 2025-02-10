use add::AddCommand;
use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use opt_in_vault::OptInVaultCommand;
use opt_out_vault::OptOutVaultCommand;
use register::RegisterCommand;
use remove::RemoveCommand;
use set_max_network_limit::SetMaxNetworkLimitCommand;
use set_middleware::SetMiddlewareCommand;
use vault_parameters::VaultParametersCommand;

mod add;
mod config;
mod consts;
mod opt_in_vault;
mod opt_out_vault;
mod register;
mod remove;
mod set_max_network_limit;
mod set_middleware;
mod utils;
mod vault_parameters;

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
    #[command(name = "opt-in-vault")]
    OptInVault(OptInVaultCommand),

    #[command(name = "opt-out-vault")]
    OptOutVault(OptOutVaultCommand),

    #[command(name = "register")]
    Register(RegisterCommand),

    #[command(name = "set-max-network-limit")]
    SetMaxNetworkLimit(SetMaxNetworkLimitCommand),

    #[command(name = "set-middleware")]
    SetMiddleware(SetMiddlewareCommand),

    #[command(name = "vault-parameters")]
    VaultParameters(VaultParametersCommand),

    // Import network management
    #[command(name = "add")]
    Add(AddCommand),

    #[command(name = "remove")]
    Remove(RemoveCommand),
}

impl NetworkCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            NetworkSubcommands::OptInVault(opt_in_vault) => {
                opt_in_vault.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::OptOutVault(opt_out_vault) => {
                opt_out_vault.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::Register(register) => {
                register.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::SetMaxNetworkLimit(set_max_network_limit) => {
                set_max_network_limit
                    .with_alias(self.alias)
                    .execute(ctx)
                    .await
            }
            NetworkSubcommands::SetMiddleware(set_middleware) => {
                set_middleware.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::VaultParameters(vault_parameters) => {
                vault_parameters.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::Add(add) => add.with_alias(self.alias).execute(ctx).await,
            NetworkSubcommands::Remove(remove) => remove.with_alias(self.alias).execute(ctx).await,
        }
    }
}
