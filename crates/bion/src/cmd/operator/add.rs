use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::EthereumOpts;

#[derive(Debug, Parser)]
pub struct AddCommand {
    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    pub address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}
