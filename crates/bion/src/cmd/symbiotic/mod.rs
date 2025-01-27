use clap::Subcommand;
use get_vault::GetVaultCommand;
use is_operator::IsOperatorCommand;
use list_vaults::ListVaultsCommand;
use network_opt_in::NetworkOptInCommand;
use network_opt_in_status::NetworkOptInStatusCommand;
use network_opt_out::NetworkOptOutCommand;
use register_operator::RegisterOperatorCommand;
use vault_opt_in::VaultOptInCommand;
use vault_opt_in_status::VaultOptInStatusCommand;
use vault_opt_out::VaultOptOutCommand;

pub mod get_vault;
pub mod is_operator;
pub mod list_vaults;
pub mod network_opt_in;
pub mod network_opt_in_status;
pub mod network_opt_out;
pub mod register_operator;
pub mod vault_opt_in;
pub mod vault_opt_in_status;
pub mod vault_opt_out;

#[derive(Debug, Subcommand)]
pub enum SymbioticCommands {
    #[command(name = "get-vault")]
    GetVault(GetVaultCommand),

    #[command(name = "is-operator")]
    IsOperator(IsOperatorCommand),

    #[command(name = "list-vaults")]
    ListVaults(ListVaultsCommand),

    #[command(name = "network-opt-in")]
    NetworkOptIn(NetworkOptInCommand),

    #[command(name = "network-opt-in-status")]
    NetworkOptInStatus(NetworkOptInStatusCommand),

    #[command(name = "network-opt-out")]
    NetworkOptOut(NetworkOptOutCommand),

    #[command(name = "register-operator")]
    RegisterOperator(RegisterOperatorCommand),

    #[command(name = "vault-opt-in")]
    VaultOptIn(VaultOptInCommand),

    #[command(name = "vault-opt-in-status")]
    VaultOptInStatus(VaultOptInStatusCommand),

    #[command(name = "vault-opt-out")]
    VaultOptOut(VaultOptOutCommand),
}
