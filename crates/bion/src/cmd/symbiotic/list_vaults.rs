use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use itertools::Itertools;
use prettytable::{row, Table};

use std::{str::FromStr, time::Instant};

use crate::{
    cmd::utils::{format_number_with_decimals, get_chain_id},
    symbiotic::utils::{fetch_token_datas, fetch_vault_addresses, fetch_vault_datas, VaultInfo},
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
        value_name = "LIMIT",
        default_value = "10",
        help = "The number of vaults to list."
    )]
    limit: u8,

    #[arg(long, help = "Only show verified vaults.", default_value = "false")]
    verified_only: bool,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListVaultsCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            limit,
            verified_only,
            eth,
        } = self;

        validate_cli_args(None, &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        {
            let txt = format!(
                "Loading vaults on chain {} with a limit of {}.",
                chain_id, limit
            );
            println!("{}", txt.as_str().bright_cyan());
            println!(
                "{}",
                "You can change this limit using --limit".bright_green()
            )
        }

        let t1 = Instant::now();
        let vault_addresses = fetch_vault_addresses(&provider, chain_id).await?;
        let total_vaults = vault_addresses.len();
        let vaults = fetch_vault_datas(&provider, chain_id, vault_addresses).await?;
        let vaults = fetch_token_datas(&provider, chain_id, vaults).await?;

        {
            let txt = format!(
                "Loaded {} vaults out of {} in {}ms",
                vaults.len(),
                total_vaults,
                t1.elapsed().as_millis()
            );
            println!("{}", txt.as_str().bright_green());
        }

        let mut table = Table::new();

        // table headers
        table.add_row(row![
            b -> "#",
            b -> "name",
            b -> "address",
            b -> "collateral_token",
            b -> "tvl",
            b -> "delegated"
        ]);

        let mut i = 0;
        for vault in vaults
            .into_iter()
            .sorted_by(|a, b| b.active_stake.cmp(&a.active_stake))
        {
            let vault_address = vault.address;
            let url = format!("{SYMBIOTIC_GITHUB_URL}/{vault_address}/{VAULT_FILE_NAME}",);
            let res = reqwest::get(&url).await?;

            let name = match res.error_for_status() {
                Ok(response) => {
                    let vault_info: VaultInfo = serde_json::from_str(&response.text().await?)?;
                    Some(vault_info.name)
                }
                _ => {
                    if self.verified_only {
                        continue;
                    }
                    None
                }
            };

            let total_stake_formatted =
                format_number_with_decimals(vault.total_stake.unwrap(), vault.decimals.unwrap())?;

            let active_stake_formatted = {
                let active = format_number_with_decimals(
                    vault.active_stake.unwrap(),
                    vault.decimals.unwrap(),
                )?;
                let total_f64 = f64::from_str(&total_stake_formatted).unwrap();
                let active_f64 = f64::from_str(&active).unwrap();
                let percentage = if total_f64 > 0.0 {
                    (active_f64 / total_f64 * 100.0).round()
                } else {
                    0.0
                };
                format!("{} ({:.0}%)", active, percentage)
            };

            let symbiotic_link = format!(
                "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
                vault_address,
                name.unwrap_or("Unverified vault".to_string())
            );
            let vault_link: String = format!(
                "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
                vault_address, vault_address
            );

            let collateral_link = format!(
                "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
                vault.collateral.unwrap(),
                vault.symbol.unwrap()
            );

            let row = row![
                i + 1,
                symbiotic_link,
                vault_link,
                collateral_link,
                total_stake_formatted,
                active_stake_formatted,
            ];

            table.add_row(row);

            i += 1;
            if i >= self.limit {
                break;
            }
        }

        table.printstd();

        Ok(())
    }
}
