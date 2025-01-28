use alloy_primitives::{utils::format_units, Address, U256};
use clap::Parser;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use itertools::Itertools;
use prettytable::{row, Cell, Table};
use serde::Deserialize;

use std::str::FromStr;

use crate::{
    symbiotic::{
        calls::{
            get_token_decimals, get_token_symbol, get_vault_active_stake, get_vault_collateral,
            get_vault_entity, get_vault_total_entities, get_vault_total_stake,
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
    #[arg(
        long,
        required = true,
        value_name = "LIMIT",
        help = "The number of vaults to list."
    )]
    limit: u8,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListVaultsCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { limit, eth } = self;

        validate_cli_args(None, &eth).await?;

        let vault_factory = Address::from_str(addresses::mainnet::VAULT_FACTORY)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let total_entities = get_vault_total_entities(vault_factory, &provider).await?;

        let mut vaults = vec![];
        for i in 0..total_entities.try_into()? {
            let vault = get_vault_entity(vault_factory, U256::try_from(i)?, &provider).await?;

            let url = format!("{SYMBIOTIC_GITHUB_URL}/{vault}/{VAULT_FILE_NAME}",);

            let mut name = None;
            let mut collateral_token = None;
            let res = reqwest::get(&url).await?;
            match res.error_for_status() {
                Ok(_res) => {
                    let vault_info: VaultInfo = serde_json::from_str(&_res.text().await?)?;
                    name = Some(vault_info.name);

                    if let Some(token) = vault_info.tags.iter().find_map(|tag| {
                        tag.strip_prefix("token:")
                            .map(|remainder| remainder.to_string())
                    }) {
                        collateral_token = Some(token);
                    }
                }
                Err(_) => {}
            }

            if let Ok(collateral_token_address) = get_vault_collateral(vault, &provider).await {
                if let Ok(decimals) = get_token_decimals(collateral_token_address, &provider).await
                {
                    if collateral_token.is_none() {
                        if let Ok(token_symbol) =
                            get_token_symbol(collateral_token_address, &provider).await
                        {
                            collateral_token = Some(token_symbol);
                        } else {
                            continue;
                        }
                    }

                    let total_stake = get_vault_total_stake(vault, &provider).await?;
                    let total_stake_formatted = format_units(total_stake, decimals)?;

                    let active_stake = get_vault_active_stake(vault, &provider).await?;
                    let active_stake_formatted = format_units(active_stake, decimals)?;

                    // let vault = format!(
                    //     "\x1b]8;;https://etherscan.io/address/{vault}\x1B\\{vault}\x1b]8;;\x1b\\"
                    // );

                    vaults.push((
                        total_stake.wrapping_div(U256::from(10).pow(U256::from(decimals))),
                        row![
                            name.unwrap_or("Unknown".to_string()),
                            vault,
                            collateral_token.unwrap_or("Unknown".to_string()),
                            total_stake_formatted,
                            active_stake_formatted,
                        ],
                    ));
                }
            }
        }

        let mut table = Table::new();

        // table headers
        table.add_row(row![
            "name",
            "address",
            "collateral_token",
            "tvl",
            "delegated"
        ]);

        let mut i = 0;
        for (_, vault) in vaults.into_iter().sorted_by(|(k1, _), (k2, _)| k2.cmp(k1)) {
            table.add_row(vault);

            i += 1;
            if i >= limit {
                break;
            }
        }

        table.printstd();

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct VaultInfo {
    name: String,
    tags: Vec<String>,
}
