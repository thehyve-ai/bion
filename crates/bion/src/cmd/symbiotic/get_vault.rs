use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::str::FromStr;

use crate::{
    common::consts::TESTNET_VAULTS,
    symbiotic::calls::{get_vault_active_stake, get_vault_total_stake},
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for Symbiotic vault.")]
pub struct GetVaultCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address of the vault."
    )]
    address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl GetVaultCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { address, eth } = self;

        validate_cli_args(None, &eth).await?;

        let mut table = Table::new();

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        // table headers
        table.add_row(row![
            "burner",
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
