use alloy_primitives::{aliases::U96, Address, U256};
use chrono::{DateTime, Utc};
use foundry_common::provider::RetryProvider;
use itertools::Itertools;
use multicall::{Multicall, MulticallVersion};
use prettytable::{row, Table};
use serde::Deserialize;

use crate::{
    cmd::utils::format_number_with_decimals,
    utils::{
        get_etherscan_address_link, parse_duration_secs, parse_epoch_ts, parse_ts,
        print_loading_until_async,
    },
};

use super::{
    calls::{
        get_delegator_type_multicall, get_max_network_limit, get_network_current_epoch,
        get_network_entity_multicall, get_network_epoch_duration_multicall,
        get_network_epoch_start_multicall, get_network_limit, get_network_middleware_multicall,
        get_network_slashing_window, get_network_total_entities, get_operator_entity_multicall,
        get_operator_total_entities, get_token_decimals_multicall, get_token_symbol_multicall,
        get_vault_collateral_multicall, get_vault_delegator_multicall, get_vault_entity_multicall,
        get_vault_total_entities, is_network, is_opted_in_network, is_opted_in_network_multicall,
    },
    consts::{
        get_network_middleware_service, get_network_opt_in_service, get_network_registry,
        get_operator_registry, get_vault_factory,
    },
    operator_utils::OperatorData,
    utils::{get_network_link, get_vault_link},
    vault_utils::VaultData,
    DelegatorType,
};

// Symbiotic network metadata
pub const SYMBIOTIC_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/symbioticfi/metadata-mainnet/refs/heads/main/networks";
pub const SYMBIOTIC_NETWORK_FILE_NAME: &str = "info.json";

#[derive(Clone, Debug)]
pub struct NetworkData {
    pub address: Address,
    pub middleware_address: Option<Address>,
    pub slashing_window: Option<U256>,
    pub current_epoch: Option<U256>,
    pub current_epoch_start: Option<U256>,
    pub epoch_duration: Option<U256>,
    pub next_epoch_start: Option<U256>,
    pub operators: Option<Vec<OperatorData>>,
    pub vaults: Option<Vec<(VaultData, U256)>>,
    pub symbiotic_metadata: Option<NetworkInfo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
}

impl NetworkData {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            middleware_address: None,
            symbiotic_metadata: None,
            current_epoch: None,
            current_epoch_start: None,
            epoch_duration: None,
            next_epoch_start: None,
            slashing_window: None,
            operators: None,
            vaults: None,
        }
    }

    pub async fn load(
        address: Address,
        chain_id: u64,
        provider: &RetryProvider,
    ) -> eyre::Result<Self> {
        let networks = fetch_network_data(chain_id, vec![address], provider).await?;
        let networks = fetch_network_operator_data(chain_id, networks, provider).await?;
        let networks = fetch_network_vault_data(chain_id, networks, provider).await?;
        let network = networks
            .first()
            .ok_or(eyre::eyre!("Network not found"))?
            .clone()
            .with_symbiotic_metadata()
            .await?;
        Ok(network)
    }

    pub fn set_middleware_address(&mut self, middleware_address: Address) {
        self.middleware_address = Some(middleware_address);
    }

    pub fn set_slashing_window(&mut self, slashing_window: U256) {
        self.slashing_window = Some(slashing_window);
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

    pub fn with_operators(mut self, operators: Vec<OperatorData>) -> Self {
        self.operators = Some(operators);
        self
    }

    pub fn with_vaults(mut self, vaults: Vec<(VaultData, U256)>) -> Self {
        self.vaults = Some(vaults);
        self
    }

    /// Loads metadata for a Symbiotic vault from the official GitHub repository
    ///
    /// # Arguments
    /// * `vault` - Struct holding the Vault's data
    ///
    /// # Returns
    /// * Empty Ok result on success
    ///
    /// # Errors
    /// * If the HTTP request fails
    /// * If the response cannot be parsed as JSON
    /// * If the JSON cannot be deserialized into `VaultInfo`
    pub async fn with_symbiotic_metadata(mut self) -> eyre::Result<Self> {
        let network_address = self.address.to_string();
        let url =
            format!("{SYMBIOTIC_GITHUB_URL}/{network_address}/{SYMBIOTIC_NETWORK_FILE_NAME}",);
        println!("Fetching network metadata from: {}", url);
        let res = reqwest::get(&url).await?;
        let mut network_info: Option<NetworkInfo> = serde_json::from_str(&res.text().await?).ok();
        if network_info.is_none() {
            let url = format!(
                "{SYMBIOTIC_GITHUB_URL}/{}/{SYMBIOTIC_NETWORK_FILE_NAME}",
                network_address.to_lowercase()
            );
            let res = reqwest::get(&url).await?;
            network_info = serde_json::from_str(&res.text().await?).ok();
        }
        self.symbiotic_metadata = network_info;
        Ok(self)
    }
}

pub struct NetworkDataTableBuilder {
    data: NetworkData,
    table: Table,
}

impl NetworkDataTableBuilder {
    pub fn from_network_data(data: NetworkData) -> Self {
        Self { data, table: Table::new() }
    }

    pub fn with_name(mut self) -> Self {
        let name = self
            .data
            .symbiotic_metadata
            .clone()
            .map(|v| v.name)
            .unwrap_or("Unverified".to_string());
        self.table.add_row(row![
            Fcb -> "Name",
            get_network_link(self.data.address, name)
        ]);
        self
    }

    pub fn with_address(mut self) -> Self {
        let link = get_etherscan_address_link(self.data.address, self.data.address.to_string());
        self.table.add_row(row![Fcb ->"Address",  link]);
        self
    }

    pub fn with_middleware(mut self) -> Self {
        let link = {
            let middleware = self.data.middleware_address.unwrap_or(Address::ZERO);
            if middleware != Address::ZERO {
                get_etherscan_address_link(middleware, middleware.to_string())
            } else {
                "-".to_string()
            }
        };
        self.table.add_row(row![Fcb ->"Middleware",  link]);
        self
    }

    pub fn with_current_epoch(mut self) -> Self {
        let current_epoch = if let Some(current_epoch) = self.data.current_epoch {
            current_epoch.to_string()
        } else {
            "-".to_string()
        };
        self.table.add_row(row![Fcb -> "Current epoch", current_epoch]);
        self
    }

    pub fn with_current_epoch_start(mut self) -> Self {
        let current_epoch_start = if let Some(current_epoch_start) = self.data.current_epoch_start {
            parse_epoch_ts(current_epoch_start)
        } else {
            "-".to_string()
        };
        self.table.add_row(row![
            Fcb -> "Current epoch start",
            current_epoch_start
        ]);
        self
    }

    pub fn with_epoch_duration(mut self) -> Self {
        let epoch_duration = if let Some(epoch_duration) = self.data.epoch_duration {
            parse_duration_secs(epoch_duration)
        } else {
            "-".to_string()
        };
        self.table.add_row(row![
            Fcb -> "Epoch duration",
            epoch_duration
        ]);
        self
    }

    pub fn with_next_epoch_start(mut self) -> Self {
        let next_epoch_start = if let Some(next_epoch_start) = self.data.next_epoch_start {
            parse_epoch_ts(next_epoch_start)
        } else {
            "-".to_string()
        };
        self.table.add_row(row![
            Fcb -> "Next epoch start",
            next_epoch_start
        ]);
        self
    }

    pub fn with_time_till_next_epoch(mut self) -> Self {
        let time_till_next_epoch = if let Some(next_epoch_start) = self.data.next_epoch_start {
            let now = Utc::now();
            let next_epoch_start =
                DateTime::<Utc>::from_timestamp(next_epoch_start.to::<i64>(), 0).unwrap();

            let time_till_next_epoch = next_epoch_start.signed_duration_since(now);
            parse_ts(time_till_next_epoch.num_seconds())
        } else {
            "-".to_string()
        };

        self.table.add_row(row![
            Fcb -> "Time till next epoch",
            time_till_next_epoch
        ]);
        self
    }

    pub fn with_slashing_window(mut self) -> Self {
        let slashing_window = if let Some(slashing_window) = self.data.slashing_window {
            parse_duration_secs(slashing_window)
        } else {
            "-".to_string()
        };

        self.table.add_row(row![
            Fcb -> "Slashing window",
            slashing_window
        ]);
        self
    }

    pub fn with_operator_count(mut self) -> Self {
        let operator_count = self.data.operators.as_ref().map(|v| v.len()).unwrap_or(0);
        self.table.add_row(row![Fcb -> "Operators",  operator_count]);
        self
    }

    pub fn with_vault_count(mut self) -> Self {
        let vault_count = self.data.vaults.as_ref().map(|v| v.len()).unwrap_or(0);
        self.table.add_row(row![Fcb -> "Vaults",  vault_count]);
        self
    }

    pub fn with_vault_subtable(mut self) -> Self {
        for (vault, network_limit) in
            self.data.vaults.as_ref().unwrap().iter().sorted_by(|a, b| b.1.cmp(&a.1))
        {
            let network_limit_formatted = if let Ok(network_limit_formatted) =
                format_number_with_decimals(*network_limit, vault.decimals.unwrap())
            {
                if network_limit_formatted == "0.000" {
                    format!("- {}", vault.symbol.clone().unwrap())
                } else {
                    format!("{} {}", network_limit_formatted, vault.symbol.clone().unwrap())
                }
            } else {
                "-".to_string()
            };
            let vault_name = vault
                .symbiotic_metadata
                .clone()
                .map(|v| v.name)
                .unwrap_or("Unverified".to_string());
            self.table.add_row(
                row![Fcb -> get_vault_link(vault.address, vault_name), network_limit_formatted],
            );
        }
        self
    }

    pub fn with_all(self) -> Self {
        self.with_name()
            .with_address()
            .with_middleware()
            .with_slashing_window()
            .with_current_epoch()
            .with_current_epoch_start()
            .with_epoch_duration()
            .with_next_epoch_start()
            .with_time_till_next_epoch()
            .with_operator_count()
            .with_vault_count()
            .with_vault_subtable()
    }

    pub fn build(self) -> Table {
        self.table
    }
}

/// Fetches metadata for a Symbiotic network from the official GitHub repository
///
/// # Arguments
/// * `network_address` - The address of the network to fetch metadata for
///
/// # Returns
/// * `NetworkInfo` containing the network's metadata
///
/// # Errors
/// * If the HTTP request fails
/// * If the response cannot be parsed as JSON
/// * If the JSON cannot be deserialized into `VaultInfo`
pub async fn get_network_metadata(network_address: Address) -> eyre::Result<Option<NetworkInfo>> {
    let network_address = network_address.to_string().to_lowercase();
    let url = format!("{SYMBIOTIC_GITHUB_URL}/{network_address}/{SYMBIOTIC_NETWORK_FILE_NAME}",);
    let res = reqwest::get(&url).await?;
    let vault_info: Option<NetworkInfo> = serde_json::from_str(&res.text().await?).ok();
    Ok(vault_info)
}

pub async fn validate_network_symbiotic_status<A: TryInto<Address>>(
    network: A,
    network_registry: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_network = print_loading_until_async(
        "Validating network Symbiotic status",
        is_network(network, network_registry, provider),
    )
    .await?;

    if !is_network {
        eyre::bail!("Network is not registered in Symbiotic");
    }

    Ok(())
}

pub async fn validate_network_opt_in_status<A: TryInto<Address>>(
    operator: A,
    network: A,
    opt_in_service: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_opted_in = print_loading_until_async(
        "Checking opted in status",
        is_opted_in_network(operator, network, opt_in_service, provider),
    )
    .await?;

    if !is_opted_in {
        eyre::bail!("Operator is not opted in network.");
    }

    Ok(())
}

pub async fn fetch_network_addresses(
    chain_id: u64,
    provider: &RetryProvider,
) -> eyre::Result<Vec<Address>> {
    let network_registry = get_network_registry(chain_id)?;

    // exclude this one from the multicall
    let total_entities =
        get_network_total_entities(network_registry, provider).await?.to::<usize>();

    let mut multicall = Multicall::with_chain_id(provider, chain_id)?;
    multicall.set_version(MulticallVersion::Multicall3);

    for i in 0..total_entities {
        get_network_entity_multicall(&mut multicall, network_registry, U256::try_from(i)?, true);
    }

    let network_addresses = multicall
        .call()
        .await?
        .into_iter()
        .filter_map(|result| match result {
            Ok(result) => result.as_address(),
            Err(_) => None,
        })
        .collect_vec();

    Ok(network_addresses)
}

pub async fn fetch_network_data(
    chain_id: u64,
    network_addresses: Vec<Address>,
    provider: &RetryProvider,
) -> eyre::Result<Vec<NetworkData>> {
    let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
    multicall.set_version(MulticallVersion::Multicall3);

    let middleware_service = get_network_middleware_service(chain_id)?;
    for network in &network_addresses {
        get_network_middleware_multicall(&mut multicall, *network, middleware_service, true);
    }

    let middleware_calls = multicall.call().await?.into_iter();

    let mut networks = Vec::with_capacity(network_addresses.len());
    for (network_data, network_address) in middleware_calls.zip(network_addresses) {
        let middleware_address = network_data.as_ref().map(|data| data.as_address()).ok().flatten();
        let mut network = NetworkData::new(network_address);
        if let Some(middleware_address) = middleware_address {
            if middleware_address != Address::ZERO {
                // Try load the slashing window
                if let Ok(slashing_window) =
                    get_network_slashing_window(middleware_address, provider).await
                {
                    network.set_slashing_window(U256::from(slashing_window));
                }

                // Try load epoch data
                if let Ok(current_epoch) =
                    get_network_current_epoch(middleware_address, provider).await
                {
                    multicall.clear_calls();
                    get_network_epoch_duration_multicall(&mut multicall, middleware_address, true);
                    get_network_epoch_start_multicall(
                        &mut multicall,
                        U256::from(current_epoch),
                        middleware_address,
                        true,
                    );
                    get_network_epoch_start_multicall(
                        &mut multicall,
                        U256::from(current_epoch) + U256::from(1),
                        middleware_address,
                        true,
                    );

                    let epoch_calls = multicall.call().await?;
                    let epoch_duration =
                        epoch_calls[0].as_ref().map(|data| data.as_uint()).ok().flatten();
                    let current_epoch_start =
                        epoch_calls[1].as_ref().map(|data| data.as_uint()).ok().flatten();
                    let next_epoch_start =
                        epoch_calls[2].as_ref().map(|data| data.as_uint()).ok().flatten();

                    network.set_current_epoch(U256::from(current_epoch));

                    if let Some(current_epoch_start) = current_epoch_start {
                        network.set_current_epoch_start(current_epoch_start.0);
                    }

                    if let Some(next_epoch_start) = next_epoch_start {
                        network.set_next_epoch_start(next_epoch_start.0);
                    }

                    if let Some(epoch_duration) = epoch_duration {
                        network.set_epoch_duration(epoch_duration.0);
                    }
                }
            }
            network.set_middleware_address(middleware_address);
        }
        network.set_middleware_address(middleware_address.unwrap_or(Address::ZERO));
        networks.push(network);
    }

    Ok(networks)
}

pub async fn fetch_network_symbiotic_metadata(
    networks: Vec<NetworkData>,
) -> eyre::Result<Vec<NetworkData>> {
    let mut out = Vec::with_capacity(networks.len());
    for network in networks {
        out.push(network.with_symbiotic_metadata().await?);
    }

    Ok(out)
}

pub async fn fetch_network_operator_data(
    chain_id: u64,
    networks: Vec<NetworkData>,
    provider: &RetryProvider,
) -> eyre::Result<Vec<NetworkData>> {
    let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
    multicall.set_version(MulticallVersion::Multicall3);

    let operator_registry = get_operator_registry(chain_id)?;
    let total_operators =
        get_operator_total_entities(operator_registry, provider).await?.to::<usize>();

    for i in 0..total_operators {
        get_operator_entity_multicall(&mut multicall, operator_registry, U256::try_from(i)?, true);
    }

    let operator_addresses = multicall
        .call()
        .await?
        .into_iter()
        .filter_map(|result| match result {
            Ok(result) => result.as_address(),
            Err(_) => None,
        })
        .collect_vec();

    multicall.clear_calls();
    let network_opt_in_service = get_network_opt_in_service(chain_id)?;
    for network in &networks {
        for operator in &operator_addresses {
            is_opted_in_network_multicall(
                &mut multicall,
                *operator,
                network.address,
                network_opt_in_service,
                true,
            );
        }
    }

    let opted_in_calls = multicall.call().await?.into_iter().chunks(operator_addresses.len());

    let mut out = Vec::with_capacity(networks.len());
    for (opted_in_call, network) in opted_in_calls.into_iter().zip(networks) {
        let mut opted_in_operators = vec![];
        for (is_opted_in, operator) in opted_in_call.into_iter().zip(operator_addresses.clone()) {
            let is_opted_in = is_opted_in.as_ref().map(|data| data.as_bool()).ok().flatten();
            if is_opted_in.unwrap_or(false) {
                let operator = OperatorData::new(operator);
                opted_in_operators.push(operator);
            }
        }
        out.push(network.with_operators(opted_in_operators));
    }

    Ok(out)
}

pub async fn fetch_network_vault_data(
    chain_id: u64,
    networks: Vec<NetworkData>,
    provider: &RetryProvider,
) -> eyre::Result<Vec<NetworkData>> {
    let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
    multicall.set_version(MulticallVersion::Multicall3);

    let vault_factory = get_vault_factory(chain_id)?;
    let total_vaults = get_vault_total_entities(vault_factory, provider).await?.to::<usize>();

    for i in 0..total_vaults {
        get_vault_entity_multicall(&mut multicall, vault_factory, U256::try_from(i)?, true);
    }

    let vault_addresses = multicall
        .call()
        .await?
        .into_iter()
        .filter_map(|result| match result {
            Ok(result) => result.as_address(),
            Err(_) => None,
        })
        .collect_vec();

    multicall.clear_calls();

    for vault in &vault_addresses {
        get_vault_collateral_multicall(&mut multicall, *vault, true);
        get_vault_delegator_multicall(&mut multicall, *vault, true);
    }

    let vault_data_calls = multicall.call().await?.into_iter().chunks(2);

    let mut vaults = vec![];
    for (vault_data_call, vault_address) in vault_data_calls.into_iter().zip(vault_addresses) {
        let vault_data_call = vault_data_call.into_iter().collect_vec();
        let collateral = vault_data_call[0].as_ref().map(|data| data.as_address()).ok().flatten();
        let delegator = vault_data_call[1].as_ref().map(|data| data.as_address()).ok().flatten();

        if collateral.is_none() || delegator.is_none() {
            continue;
        }

        let collateral = collateral.unwrap();
        let delegator = delegator.unwrap();
        let mut vault = VaultData::new(vault_address);
        vault.set_collateral(collateral);
        vault.set_delegator(delegator);
        vaults.push(vault);
    }

    multicall.clear_calls();

    for vault in &vaults {
        get_delegator_type_multicall(&mut multicall, vault.delegator.unwrap(), true);
        get_token_decimals_multicall(&mut multicall, vault.collateral.unwrap(), true);
        get_token_symbol_multicall(&mut multicall, vault.collateral.unwrap(), true);
    }

    let vault_data_calls = multicall.call().await?.into_iter().chunks(3);

    let mut out = vec![];
    for network in networks {
        let mut opted_in_vaults = vec![];
        for (vault_data_call, vault) in vault_data_calls.into_iter().zip(vaults.iter()) {
            let vault_data_call = vault_data_call.into_iter().collect_vec();
            let delegator_type =
                vault_data_call[0].as_ref().map(|data| data.as_uint()).ok().flatten();

            let delegator_type: u64 = delegator_type.unwrap().0.try_into()?;
            let network_limit = if DelegatorType::from(delegator_type)
                == DelegatorType::OperatorNetworkSpecificDelegator
            {
                get_max_network_limit(
                    network.address,
                    U96::ZERO,
                    vault.delegator.unwrap(),
                    provider,
                )
                .await?
            } else {
                get_network_limit(network.address, U96::ZERO, vault.delegator.unwrap(), provider)
                    .await?
            };

            if network_limit != U256::ZERO {
                let decimals =
                    vault_data_call[1].as_ref().map(|data| data.as_uint()).ok().flatten();
                let symbol = vault_data_call[2].as_ref().map(|data| data.as_str()).ok().flatten();
                if decimals.is_none() || symbol.is_none() {
                    continue;
                }

                let decimals = decimals.unwrap();
                let symbol = symbol.unwrap();

                let mut opted_in_vault = vault.clone().with_symbiotic_metadata().await?;
                opted_in_vault.set_decimals(decimals.0.try_into()?);
                opted_in_vault.set_symbol(symbol.to_string());
                opted_in_vaults.push((opted_in_vault, network_limit));
            }
        }

        out.push(network.with_vaults(opted_in_vaults));
    }

    Ok(out)
}
