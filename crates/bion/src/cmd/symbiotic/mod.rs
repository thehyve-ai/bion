use clap::Subcommand;
use network::NetworkCommands;
use operator::OperatorCommands;
use vault::VaultCommands;

pub mod network;
pub mod operator;
pub mod vault;

#[derive(Debug, Subcommand)]
pub enum SymbioticCommands {
    #[command(
        name = "network",
        about = "Commands for network general use.",
        subcommand
    )]
    Network(NetworkCommands),

    #[command(
        name = "operator",
        about = "Commands for operator general use.",
        subcommand
    )]
    Operator(OperatorCommands),

    #[command(name = "vault", about = "Commands for vault general use.", subcommand)]
    Vault(VaultCommands),
}
