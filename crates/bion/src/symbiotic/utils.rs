use alloy_primitives::{Address, U256};
use colored::Colorize;
use foundry_common::provider::RetryProvider;
use itertools::Itertools;
use multicall::{Multicall, MulticallVersion};
use serde::Deserialize;

use std::str::FromStr;

use crate::cmd::utils::format_number_with_decimals;

use super::calls::{
    get_token_decimals_multicall, get_token_symbol_multicall, get_vault_active_stake_multicall,
    get_vault_collateral_multicall, get_vault_delegator_multicall, get_vault_total_stake_multicall,
};

const SYMBIOTIC_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/symbioticfi/metadata-mainnet/refs/heads/main/vaults";
const VAULT_FILE_NAME: &str = "info.json";

#[derive(Clone)]
pub struct VaultData {
    pub address: Address,
    pub name: Option<String>,
    pub collateral: Option<Address>,
    pub delegator: Option<Address>,
    pub total_stake: Option<U256>,
    pub active_stake: Option<U256>,
    pub decimals: Option<u8>,
    pub symbol: Option<String>,
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

    pub fn total_stake_formatted(&self) -> Option<String> {
        if self.total_stake.is_none() || self.decimals.is_none() {
            return None;
        }
        format_number_with_decimals(self.total_stake.unwrap(), self.decimals.unwrap()).ok()
    }

    pub fn active_stake_formatted(&self) -> Option<String> {
        if self.active_stake.is_none() || self.decimals.is_none() {
            return None;
        }
        format_number_with_decimals(self.active_stake.unwrap(), self.decimals.unwrap()).ok()
    }

    pub fn active_stake_formatted_with_percentage(&self) -> Option<String> {
        if self.total_stake.is_none() || self.active_stake.is_none() || self.decimals.is_none() {
            return None;
        }

        let total_stake = self.total_stake_formatted()?;
        let active_stake = self.active_stake_formatted()?;

        let total_f64 = f64::from_str(&total_stake).unwrap();
        let active_f64 = f64::from_str(&active_stake).unwrap();

        let percentage = if total_f64 > 0.0 {
            (active_f64 / total_f64 * 100.0).round()
        } else {
            0.0
        };
        Some(format!("{} ({:.0}%)", active_stake, percentage))
    }
}

/// Fetches data for multiple vaults using multicall
///
/// # Arguments
/// * `provider` - The Ethereum provider to use for making calls
/// * `chain_id` - The chain ID to use for multicall
/// * `vaults_addresses` - Vector of vault addresses to fetch data for
///
/// # Returns
/// A vector of `VaultData` structs containing the fetched vault information
///
/// # Errors
/// Returns an error if:
/// - Multicall setup fails
/// - Any of the vault data calls fail
pub async fn fetch_vault_datas(
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

/// Fetches token metadata (decimals and symbol) for a list of vaults' collateral tokens
///
/// # Arguments
///
/// * `provider` - The Ethereum provider to use for making RPC calls
/// * `chain_id` - The chain ID to use for the multicall contract
/// * `vaults` - List of vaults to fetch token data for
///
/// # Returns
///
/// Returns a filtered list of vaults with their collateral token metadata populated.
/// Vaults whose tokens fail to return valid decimals or symbols are skipped.
pub async fn fetch_token_datas(
    provider: &RetryProvider,
    chain_id: u64,
    vaults: Vec<VaultData>,
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

#[derive(Debug, Deserialize)]
pub struct VaultInfo {
    pub name: String,
    pub tags: Vec<String>,
}

/// Fetches metadata for a Symbiotic vault from the official GitHub repository
///
/// # Arguments
/// * `vault_address` - The address of the vault to fetch metadata for
///
/// # Returns
/// * `VaultInfo` containing the vault's name and tags
///
/// # Errors
/// * If the HTTP request fails
/// * If the response cannot be parsed as JSON
/// * If the JSON cannot be deserialized into `VaultInfo`
pub async fn get_vault_metadata(vault_address: Address) -> eyre::Result<Option<VaultInfo>> {
    let url = format!("{SYMBIOTIC_GITHUB_URL}/{vault_address}/{VAULT_FILE_NAME}",);
    let res = reqwest::get(&url).await?;
    let vault_info: Option<VaultInfo> = serde_json::from_str(&res.text().await?).ok();
    Ok(vault_info)
}
