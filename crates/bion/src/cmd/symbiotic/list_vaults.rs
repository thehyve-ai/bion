use alloy_primitives::{Address, U256};
use clap::Parser;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};
use serde::Deserialize;

use std::str::FromStr;

use crate::{
    symbiotic::{
        calls::{
            get_vault_active_stake, get_vault_entity, get_vault_total_entities,
            get_vault_total_stake,
        },
        consts::addresses,
    },
    utils::validate_cli_args,
};

const SYMBIOTIC_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/symbioticfi/metadata-mainnet/refs/heads/main/vaults";
const VAULT_FILE_NAME: &str = "info.json";

#[derive(Debug, Parser)]
#[clap(about = "Get information for all Symbiotic vaults.")]
pub struct ListVaultsCommand {
    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListVaultsCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { eth } = self;

        validate_cli_args(None, &eth).await?;

        let mut table = Table::new();

        // table headers
        table.add_row(row!["address", "collateral_token", "tvl", "delegated"]);

        let vault_factory = Address::from_str(addresses::mainnet::VAULT_FACTORY)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let total_entities = get_vault_total_entities(vault_factory, &provider).await?;

        for i in 0..total_entities.try_into()? {
            let vault = get_vault_entity(vault_factory, U256::try_from(i)?, &provider).await?;

            let url = format!("{SYMBIOTIC_GITHUB_URL}/{vault}/{VAULT_FILE_NAME}",);

            let resp = reqwest::get(&url).await?.text().await?;
            let vault_info: VaultInfo = serde_json::from_str(&resp)?;

            let total_stake = get_vault_total_stake(vault, &provider).await?;

            let active_stake = get_vault_active_stake(vault, &provider).await?;

            table.add_row(row![vault, vault_info.name, total_stake, active_stake,]);
        }

        table.printstd();

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct VaultInfo {
    name: String,
}
