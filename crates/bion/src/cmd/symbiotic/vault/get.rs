use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::str::FromStr;

use crate::{
    common::consts::TESTNET_VAULTS,
    symbiotic::calls::{get_vault_active_stake, get_vault_total_stake},
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for a single vault in Symbiotic.")]
pub struct GetCommand {
    #[clap(flatten)]
    eth: EthereumOpts,

    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address of the vault."
    )]
    address: Address,
}

impl GetCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let mut table = Table::new();

        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        // table headers
        table.add_row(row![
            "alias",
            "address",
            "collateral_token",
            "tvl",
            "delegated"
        ]);

        for (token, vault_address) in TESTNET_VAULTS.entries() {
            let active_stake =
                get_vault_active_stake(Address::from_str(&vault_address)?, &provider).await?;

            let total_stake =
                get_vault_total_stake(Address::from_str(&vault_address)?, &provider).await?;

            table.add_row(row![
                "None",
                vault_address,
                token,
                total_stake,
                active_stake,
            ]);
        }

        table.printstd();

        Ok(())
    }
}
