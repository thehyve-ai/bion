use clap::Subcommand;
use get_vault::GetVaultCommand;
use list_vaults::ListVaultsCommand;

mod get_vault;
mod list_vaults;

#[derive(Debug, Subcommand)]
#[clap(about = "General use commands for the symbiotic network.")]
pub enum SymbioticCommands {
    #[command(name = "get-vault")]
    GetVault(GetVaultCommand),

    #[command(name = "list-vaults")]
    ListVaults(ListVaultsCommand),
}
