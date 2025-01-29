use alloy_primitives::{utils::format_units, Address, U256};
use cast::Cast;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use foundry_common::provider::RetryProvider;
use hyve_cli_runner::CliContext;
use itertools::Itertools;
use multicall::{Multicall, MulticallVersion};
use prettytable::{cell, format::TableFormat, row, Cell, Table};
use serde::Deserialize;

use std::{collections::BTreeMap, str::FromStr, time::Instant};

use crate::{
    symbiotic::{
        calls::{
            get_token_decimals, get_token_decimals_multicall, get_token_symbol,
            get_token_symbol_multicall, get_vault_active_stake, get_vault_active_stake_multicall,
            get_vault_collateral, get_vault_collateral_multicall, get_vault_delegator_multicall,
            get_vault_entity, get_vault_entity_multicall, get_vault_total_entities,
            get_vault_total_stake, get_vault_total_stake_multicall,
        },
        consts::{addresses, get_vault_factory},
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

struct VaultData {
    address: Address,
    name: Option<String>,
    collateral: Option<Address>,
    delegator: Option<Address>,
    total_stake: Option<U256>,
    active_stake: Option<U256>,
    decimals: Option<u8>,
    symbol: Option<String>,
}

impl VaultData {
    fn new(address: Address) -> Self {
        Self {
            address,
            name: None,
            collateral: None,
            delegator: None,
            total_stake: None,
            active_stake: None,
            decimals: None,
            symbol: None,
        }
    }

    fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn set_collateral(&mut self, collateral: Address) {
        self.collateral = Some(collateral);
    }

    fn set_delegator(&mut self, delegator: Address) {
        self.delegator = Some(delegator);
    }

    fn set_total_stake(&mut self, total_stake: U256) {
        self.total_stake = Some(total_stake);
    }

    fn set_active_stake(&mut self, active_stake: U256) {
        self.active_stake = Some(active_stake);
    }

    fn set_decimals(&mut self, decimals: u8) {
        self.decimals = Some(decimals);
    }

    fn set_symbol(&mut self, symbol: String) {
        self.symbol = Some(symbol);
    }
}

impl ListVaultsCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        validate_cli_args(None, &self.eth).await?;

        let config = self.eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = self.get_chain_id(&provider).await?;
        {
            let txt = format!(
                "Loading vaults on chain {} with a limit of {}.",
                chain_id, self.limit
            );
            println!("{}", txt.as_str().bright_cyan());
            println!(
                "{}",
                "You can change this limit using --limit".bright_green()
            )
        }

        let t1 = Instant::now();
        let vault_addresses = self.fetch_vault_addresses(&provider, chain_id).await?;
        let total_vaults = vault_addresses.len();
        let vaults = self
            .fetch_vault_datas(&provider, chain_id, vault_addresses)
            .await?;
        let vaults = self.fetch_token_datas(&provider, chain_id, vaults).await?;

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

    async fn get_chain_id(&self, provider: &RetryProvider) -> eyre::Result<u64> {
        // get the chain id
        let cast = Cast::new(&provider);
        let chain_id = cast.chain_id().await?;

        Ok(chain_id)
    }

    async fn fetch_vault_addresses(
        &self,
        provider: &RetryProvider,
        chain_id: u64,
    ) -> eyre::Result<Vec<Address>> {
        let vault_factory = get_vault_factory(chain_id)?;

        // exclude this one from the multicall
        let total_entities = get_vault_total_entities(vault_factory, &provider)
            .await?
            .to::<usize>();

        let mut multicall = Multicall::with_chain_id(provider, chain_id)?;
        multicall.set_version(MulticallVersion::Multicall3);

        // We first need all of the vaults to get the other data
        for i in 0..total_entities {
            get_vault_entity_multicall(&mut multicall, vault_factory, U256::try_from(i)?, true);
        }

        let vaults_addresses = multicall
            .call()
            .await?
            .into_iter()
            .filter_map(|result| match result {
                Ok(result) => result.as_address(),
                Err(_) => None,
            })
            .collect_vec();

        Ok(vaults_addresses)
    }

    async fn fetch_vault_datas(
        &self,
        provider: &RetryProvider,
        chain_id: u64,
        vaults_addresses: Vec<Address>,
    ) -> eyre::Result<Vec<VaultData>> {
        let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
        multicall.set_version(MulticallVersion::Multicall3);

        // now we can do more in bulk for vault parameters
        for vault in &vaults_addresses {
            get_vault_collateral_multicall(&mut multicall, *vault, true);
            get_vault_delegator_multicall(&mut multicall, *vault, true);
            get_vault_total_stake_multicall(&mut multicall, *vault, true);
            get_vault_active_stake_multicall(&mut multicall, *vault, true);
        }

        let vault_datas = multicall.call().await?.into_iter().chunks(4);

        let mut vaults = Vec::with_capacity(vaults_addresses.len());
        for (vault_data, vault) in vault_datas.into_iter().zip(vaults_addresses) {
            let vault_data = vault_data.into_iter().collect_vec();
            let collateral = vault_data[0]
                .as_ref()
                .map(|data| data.as_address())
                .ok()
                .flatten();
            let delegator = vault_data[1]
                .as_ref()
                .map(|data| data.as_address())
                .ok()
                .flatten();
            let total_stake = vault_data[2]
                .as_ref()
                .map(|data| data.as_uint())
                .ok()
                .flatten();
            let active_stake = vault_data[3]
                .as_ref()
                .map(|data| data.as_uint())
                .ok()
                .flatten();

            // Skip if any of the vault data is missing/errored
            if collateral.is_none()
                || delegator.is_none()
                || total_stake.is_none()
                || active_stake.is_none()
            {
                println!("{} {}", "Skipping vault: ".bright_yellow(), vault);
                continue;
            }

            let collateral = collateral.unwrap();
            let delegator = delegator.unwrap();
            let total_stake = total_stake.unwrap();
            let active_stake = active_stake.unwrap();

            let mut vault_data = VaultData::new(vault);
            vault_data.set_collateral(collateral);
            vault_data.set_delegator(delegator);
            vault_data.set_total_stake(total_stake.0);
            vault_data.set_active_stake(active_stake.0);

            vaults.push(vault_data);
        }
        Ok(vaults)
    }

    async fn fetch_token_datas(
        &self,
        provider: &RetryProvider,
        chain_id: u64,
        mut vaults: Vec<VaultData>,
    ) -> eyre::Result<Vec<VaultData>> {
        let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
        multicall.set_version(MulticallVersion::Multicall3);

        for vault in &vaults {
            get_token_decimals_multicall(&mut multicall, vault.collateral.clone().unwrap(), true);
            get_token_symbol_multicall(&mut multicall, vault.collateral.clone().unwrap(), true);
        }

        let token_calls = multicall.call().await?.into_iter().chunks(2);

        let mut out = Vec::with_capacity(vaults.len());
        for (token_call, mut vault) in token_calls.into_iter().zip(vaults) {
            let token_call = token_call.into_iter().collect_vec();
            let decimals = token_call[0]
                .as_ref()
                .map(|data| data.as_uint())
                .ok()
                .flatten();
            let symbol = token_call[1]
                .as_ref()
                .map(|data| data.as_str())
                .ok()
                .flatten();

            if decimals.is_none() || symbol.is_none() {
                println!("{} {}", "Skipping vault: ".bright_yellow(), vault.address);
                continue;
            }

            let decimals = decimals.unwrap();
            let symbol = symbol.unwrap();

            vault.set_decimals(decimals.0.try_into()?);
            vault.set_symbol(symbol.to_string());
            out.push(vault);
        }

        Ok(out)
    }
}

#[derive(Debug, Deserialize)]
struct VaultInfo {
    name: String,
    tags: Vec<String>,
}

fn format_number_with_decimals(value: U256, decimals: u8) -> eyre::Result<String> {
    let num = format_units(value, decimals)?;
    let num: f64 = num.parse()?;
    if num < 10.0 {
        Ok(format!("{:.3}", num))
    } else {
        Ok(format!("{:.2}", num))
    }
}
