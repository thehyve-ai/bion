use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use list_vaults::ListVaultsCommand;
use register::RegisterCommand;
use set_max_network_limit::SetMaxNetworkLimitCommand;
use set_middleware::SetMiddlewareCommand;
use set_resolver::SetResolverCommand;
use status::StatusCommand;
use vault_parameters::VaultParametersCommand;

mod list_vaults;
mod register;
mod set_max_network_limit;
mod set_middleware;
mod set_resolver;
mod status;
mod vault_parameters;

#[derive(Debug, Parser)]
#[clap(about = "Commands related to registering and managing Networks.")]
pub struct NetworkCommand {
    #[arg(value_name = "ALIAS", help = "The saved network alias.")]
    alias: String,

    #[command(subcommand)]
    pub command: NetworkSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum NetworkSubcommands {
    #[command(name = "list-vaults")]
    ListVaults(ListVaultsCommand),

    #[command(name = "register")]
    Register(RegisterCommand),

    #[command(name = "set-max-network-limit")]
    SetMaxNetworkLimit(SetMaxNetworkLimitCommand),

    #[command(name = "set-middleware")]
    SetMiddleware(SetMiddlewareCommand),

    #[command(name = "set-resolver")]
    SetResolver(SetResolverCommand),

    #[command(name = "status")]
    Status(StatusCommand),

    #[command(name = "vault-parameters")]
    VaultParameters(VaultParametersCommand),
}

impl NetworkCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            NetworkSubcommands::ListVaults(list_vaults) => {
                list_vaults.with_alias(self.alias).execute(ctx).await
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
            NetworkSubcommands::SetResolver(set_resolver) => {
                set_resolver.with_alias(self.alias).execute(ctx).await
            }
            NetworkSubcommands::Status(status) => status.with_alias(self.alias).execute(ctx).await,
            NetworkSubcommands::VaultParameters(vault_parameters) => {
                vault_parameters.with_alias(self.alias).execute(ctx).await
            }
        }
    }
}
