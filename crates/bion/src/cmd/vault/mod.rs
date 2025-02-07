use add_vault_admin::AddVaultAdminCommand;
use clap::{Parser, Subcommand};
use create::CreateCommand;
use create_burner_router::CreateBurnerRouterCommand;
use hyve_cli_runner::CliContext;
use remove_vault_admin::RemoveVaultAdminCommand;
use set_delegator::SetDelegatorCommand;
use set_deposit_limit::SetDepositLimitCommand;
use set_deposit_whitelist::SetDepositWhitelistCommand;
use set_depositor_whitelist_status::SetDepositorWhitelistStatusCommand;
use set_is_deposit_limit::SetIsDepositLimitCommand;
use set_slasher::SetSlasherCommand;

mod add_vault_admin;
pub mod config;
mod consts;
mod create;
mod create_burner_router;
mod remove_vault_admin;
mod set_delegator;
mod set_deposit_limit;
mod set_deposit_whitelist;
mod set_depositor_whitelist_status;
mod set_is_deposit_limit;
mod set_slasher;
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

    #[command(name = "set-delegator")]
    SetDelegator(SetDelegatorCommand),

    #[command(name = "set-deposit-limit")]
    SetDepositLimit(SetDepositLimitCommand),

    #[command(name = "set-deposit-whitelist")]
    SetDepositWhitelist(SetDepositWhitelistCommand),

    #[command(name = "set-depositor-whitelist-status")]
    SetDepositorWhitelistStatus(SetDepositorWhitelistStatusCommand),

    #[command(name = "set-is-deposit-limit")]
    SetIsDepositLimit(SetIsDepositLimitCommand),

    #[command(name = "set-slasher")]
    SetSlasher(SetSlasherCommand),

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
            VaultSubcommands::SetDelegator(set_delegator) => {
                set_delegator.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetDepositLimit(set_deposit_limit) => {
                set_deposit_limit.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetDepositWhitelist(set_deposit_whitelist) => {
                set_deposit_whitelist
                    .with_alias(self.alias)
                    .execute(ctx)
                    .await
            }
            VaultSubcommands::SetDepositorWhitelistStatus(set_depositor_whitelist_status) => {
                set_depositor_whitelist_status
                    .with_alias(self.alias)
                    .execute(ctx)
                    .await
            }
            VaultSubcommands::SetIsDepositLimit(set_is_deposit_limit) => {
                set_is_deposit_limit
                    .with_alias(self.alias)
                    .execute(ctx)
                    .await
            }
            VaultSubcommands::SetSlasher(set_slasher) => {
                set_slasher.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::AddVaultAdmin(add) => add.with_alias(self.alias).execute(ctx).await,
            VaultSubcommands::RemoveVaultAdmin(remove) => {
                remove.with_alias(self.alias).execute(ctx).await
            }
        }
    }
}
