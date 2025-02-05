use add_vault_admin::AddVaultAdminCommand;
use clap::{Parser, Subcommand};
use create::CreateCommand;
use create_burner_router::CreateBurnerRouterCommand;
use hyve_cli_runner::CliContext;
use remove_vault_admin::RemoveVaultAdminCommand;

mod add_vault_admin;
mod config;
mod consts;
mod create;
mod create_burner_router;
mod remove_vault_admin;
mod utils;

#[derive(Debug, Parser)]
#[clap(about = "Commands related to creating and managing Vaults on Symbiotic.")]
pub struct VaultCommand {
    #[arg(value_name = "ALIAS", help = "The saved operator alias.")]
    pub alias: String,

    #[command(subcommand)]
    pub command: VaultSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum VaultSubcommands {
    #[command(name = "create")]
    Create(CreateCommand),

    #[command(name = "create-burner-router")]
    CreateBurnerRouter(CreateBurnerRouterCommand),

    // Import vault management
    #[command(name = "add-vault-admin")]
    AddVaultAdmin(AddVaultAdminCommand),

    #[command(name = "remove-vault-admin")]
    RemoveVaultAdmin(RemoveVaultAdminCommand),
}

impl VaultCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            VaultSubcommands::Create(create) => create.with_alias(self.alias).execute(ctx).await,
            VaultSubcommands::CreateBurnerRouter(opt_in_network) => {
                opt_in_network.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::AddVaultAdmin(add) => add.with_alias(self.alias).execute(ctx).await,
            VaultSubcommands::RemoveVaultAdmin(remove) => {
                remove.with_alias(self.alias).execute(ctx).await
            }
        }
    }
}
