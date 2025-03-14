use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use network_parameters::NetworkParametersCommand;
use set_delegator::SetDelegatorCommand;
use set_deposit_limit::SetDepositLimitCommand;
use set_deposit_whitelist::SetDepositWhitelistCommand;
use set_depositor_whitelist_status::SetDepositorWhitelistStatusCommand;
use set_is_deposit_limit::SetIsDepositLimitCommand;
use set_network_limit::SetNetworkLimitCommand;
use set_operator_network_limit::SetOperatorNetworkLimitCommand;
use set_operator_network_shares::SetOperatorNetworkSharesCommand;
use set_slasher::SetSlasherCommand;

mod network_parameters;
mod set_delegator;
mod set_deposit_limit;
mod set_deposit_whitelist;
mod set_depositor_whitelist_status;
mod set_is_deposit_limit;
mod set_network_limit;
mod set_operator_network_limit;
mod set_operator_network_shares;
mod set_slasher;

#[derive(Debug, Parser)]
#[clap(about = "Commands related to creating and managing Vaults.")]
pub struct VaultCommand {
    #[arg(value_name = "ALIAS", help = "The saved operator alias.")]
    alias: String,

    #[command(subcommand)]
    pub command: VaultSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum VaultSubcommands {
    #[command(name = "network-parameters")]
    NetworkParameters(NetworkParametersCommand),

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

    #[command(name = "set-network-limit")]
    SetNetworkLimit(SetNetworkLimitCommand),

    #[command(name = "set-operator-network-limit")]
    SetOperatorNetworkLimit(SetOperatorNetworkLimitCommand),

    #[command(name = "set-operator-network-shares")]
    SetOperatorNetworkShares(SetOperatorNetworkSharesCommand),

    #[command(name = "set-slasher")]
    SetSlasher(SetSlasherCommand),
}

impl VaultCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            VaultSubcommands::NetworkParameters(network_parameters) => {
                network_parameters.execute(ctx).await
            }
            VaultSubcommands::SetDelegator(set_delegator) => {
                set_delegator.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetDepositLimit(set_deposit_limit) => {
                set_deposit_limit.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetDepositWhitelist(set_deposit_whitelist) => {
                set_deposit_whitelist.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetDepositorWhitelistStatus(set_depositor_whitelist_status) => {
                set_depositor_whitelist_status.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetIsDepositLimit(set_is_deposit_limit) => {
                set_is_deposit_limit.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetNetworkLimit(set_network_limit) => {
                set_network_limit.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetOperatorNetworkLimit(set_operator_network_limit) => {
                set_operator_network_limit.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetOperatorNetworkShares(set_operator_network_shares) => {
                set_operator_network_shares.with_alias(self.alias).execute(ctx).await
            }
            VaultSubcommands::SetSlasher(set_slasher) => {
                set_slasher.with_alias(self.alias).execute(ctx).await
            }
        }
    }
}
