use alloy_primitives::{Address, U256};
use colored::Colorize;
use foundry_common::provider::RetryProvider;
use itertools::Itertools;
use multicall::{Multicall, MulticallVersion};
use serde::Deserialize;

use std::str::FromStr;

use crate::cmd::utils::{format_number_with_decimals, parse_currency};

use super::{
    calls::{
        get_token_decimals_multicall, get_token_symbol_multicall, get_vault_active_stake_multicall,
        get_vault_burner_multicall, get_vault_collateral_multicall,
        get_vault_current_epoch_multicall, get_vault_current_epoch_start_multicall,
        get_vault_delegator_multicall, get_vault_deposit_limit_multicall,
        get_vault_deposit_whitelist_multicall, get_vault_entity_multicall,
        get_vault_epoch_duration_multicall, get_vault_next_epoch_start_multicall,
        get_vault_slasher_multicall, get_vault_total_entities, get_vault_total_stake_multicall,
    },
    consts::get_vault_factory,
};

const SYMBIOTIC_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/symbioticfi/metadata-mainnet/refs/heads/main/vaults";
const VAULT_FILE_NAME: &str = "info.json";

#[derive(Clone)]
pub struct VaultData {
    pub address: Address,
    pub collateral: Option<Address>,
    pub delegator: Option<Address>,
    pub total_stake: Option<U256>,
    pub active_stake: Option<U256>,
    pub decimals: Option<u8>,
    pub symbol: Option<String>,
    pub slasher: Option<Address>,
    pub burner: Option<Address>,
    pub deposit_limit: Option<U256>,
    pub deposit_whitelist: Option<bool>,
    pub current_epoch: Option<U256>,
    pub current_epoch_start: Option<U256>,
    pub epoch_duration: Option<U256>,
    pub next_epoch_start: Option<U256>,
}

impl VaultData {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            collateral: None,
            delegator: None,
            total_stake: None,
            active_stake: None,
            decimals: None,
            symbol: None,
            slasher: None,
            burner: None,
            deposit_limit: None,
            deposit_whitelist: None,
            current_epoch: None,
            current_epoch_start: None,
            epoch_duration: None,
            next_epoch_start: None,
        }
    }

    pub fn set_collateral(&mut self, collateral: Address) {
        self.collateral = Some(collateral);
    }

    pub fn set_delegator(&mut self, delegator: Address) {
        self.delegator = Some(delegator);
    }

    pub fn set_total_stake(&mut self, total_stake: U256) {
        self.total_stake = Some(total_stake);
    }

    pub fn set_active_stake(&mut self, active_stake: U256) {
        self.active_stake = Some(active_stake);
    }

    pub fn set_decimals(&mut self, decimals: u8) {
        self.decimals = Some(decimals);
    }

    pub fn set_symbol(&mut self, symbol: String) {
        self.symbol = Some(symbol);
    }

    pub fn set_slasher(&mut self, slasher: Address) {
        self.slasher = Some(slasher);
    }

    pub fn set_burner(&mut self, burner: Address) {
        self.burner = Some(burner);
    }

    pub fn set_deposit_limit(&mut self, deposit_limit: U256) {
        self.deposit_limit = Some(deposit_limit);
    }

    pub fn set_deposit_whitelist(&mut self, deposit_whitelist: bool) {
        self.deposit_whitelist = Some(deposit_whitelist);
    }

    pub fn set_current_epoch(&mut self, current_epoch: U256) {
        self.current_epoch = Some(current_epoch);
    }

    pub fn set_current_epoch_start(&mut self, current_epoch_start: U256) {
        self.current_epoch_start = Some(current_epoch_start);
    }

    pub fn set_epoch_duration(&mut self, epoch_duration: U256) {
        self.epoch_duration = Some(epoch_duration);
    }

    pub fn set_next_epoch_start(&mut self, next_epoch_start: U256) {
        self.next_epoch_start = Some(next_epoch_start);
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

        let total_f64 = parse_currency(self.total_stake.unwrap(), self.decimals.unwrap()).unwrap();
        let active_f64 =
            parse_currency(self.active_stake.unwrap(), self.decimals.unwrap()).unwrap();

        let active_stake = self.active_stake_formatted()?;

        let percentage = if total_f64 > 0.0 {
            (active_f64 / total_f64 * 100.0).round()
        } else {
            0.0
        };
        Some(format!("{} ({:.0}%)", active_stake, percentage))
    }

    pub fn deposit_limit_formatted(&self) -> Option<String> {
        if self.deposit_limit.is_none() || self.decimals.is_none() {
            return None;
        }
        format_number_with_decimals(self.deposit_limit.unwrap(), self.decimals.unwrap()).ok()
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

pub async fn fetch_vault_addresses(
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

pub async fn fetch_vault_extra_metadata(
    provider: &RetryProvider,
    chain_id: u64,
    vaults: Vec<VaultData>,
) -> eyre::Result<Vec<VaultData>> {
    let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
    multicall.set_version(MulticallVersion::Multicall3);

    for vault in &vaults {
        get_vault_slasher_multicall(&mut multicall, vault.address, false);
        get_vault_burner_multicall(&mut multicall, vault.address, false);
        get_vault_deposit_limit_multicall(&mut multicall, vault.address, false);
        get_vault_deposit_whitelist_multicall(&mut multicall, vault.address, false);
        get_vault_current_epoch_multicall(&mut multicall, vault.address, false);
        get_vault_current_epoch_start_multicall(&mut multicall, vault.address, false);
        get_vault_epoch_duration_multicall(&mut multicall, vault.address, false);
        get_vault_next_epoch_start_multicall(&mut multicall, vault.address, false);
    }

    let extra_metadata_calls = multicall.call().await?.into_iter().chunks(8);

    let mut out = Vec::with_capacity(vaults.len());
    for (extra_metadata_call, mut vault) in extra_metadata_calls.into_iter().zip(vaults) {
        let vault_call = extra_metadata_call.into_iter().collect_vec();
        let slasher = vault_call[0]
            .as_ref()
            .map(|data| data.as_address())
            .ok()
            .flatten();
        let burner = vault_call[1]
            .as_ref()
            .map(|data| data.as_address())
            .ok()
            .flatten();
        let deposit_limit = vault_call[2]
            .as_ref()
            .map(|data| data.as_uint())
            .ok()
            .flatten();
        let deposit_whitelist = vault_call[3]
            .as_ref()
            .map(|data| data.as_bool())
            .ok()
            .flatten();
        let current_epoch = vault_call[4]
            .as_ref()
            .map(|data| data.as_uint())
            .ok()
            .flatten();
        let current_epoch_start = vault_call[5]
            .as_ref()
            .map(|data| data.as_uint())
            .ok()
            .flatten();
        let epoch_duration = vault_call[6]
            .as_ref()
            .map(|data| data.as_uint())
            .ok()
            .flatten();
        let next_epoch_start = vault_call[7]
            .as_ref()
            .map(|data| data.as_uint())
            .ok()
            .flatten();

        if let Some(slasher) = slasher {
            vault.set_slasher(slasher);
        }
        if let Some(burner) = burner {
            vault.set_burner(burner);
        }
        if let Some(deposit_limit) = deposit_limit {
            vault.set_deposit_limit(deposit_limit.0);
        }
        if let Some(deposit_whitelist) = deposit_whitelist {
            vault.set_deposit_whitelist(deposit_whitelist);
        }
        if let Some(current_epoch) = current_epoch {
            vault.set_current_epoch(current_epoch.0);
        }
        if let Some(current_epoch_start) = current_epoch_start {
            vault.set_current_epoch_start(current_epoch_start.0);
        }
        if let Some(epoch_duration) = epoch_duration {
            vault.set_epoch_duration(epoch_duration.0);
        }
        if let Some(next_epoch_start) = next_epoch_start {
            vault.set_next_epoch_start(next_epoch_start.0);
        }
        out.push(vault);
    }

    Ok(out)
}

#[derive(Debug, Deserialize)]
pub struct VaultInfo {
    pub name: String,
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
