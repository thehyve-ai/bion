use alloy_primitives::{
    aliases::{U48, U96},
    hex::ToHexExt,
    Address, Bytes, U256,
};
use alloy_sol_types::SolValue;
use chrono::{DateTime, Utc};
use colored::Colorize;
use foundry_common::provider::RetryProvider;
use itertools::Itertools;
use multicall::{Multicall, MulticallVersion};
use prettytable::{row, Table};
use serde::Deserialize;

use crate::{
    cmd::{
        alias_config::AliasConfig,
        utils::{format_number_with_decimals, parse_currency},
    },
    utils::{
        get_etherscan_address_link, parse_duration_secs, parse_epoch_ts, parse_ts,
        print_error_message, print_loading_until_async,
    },
};

use super::{
    calls::{
        get_network_limit, get_token_decimals_multicall, get_token_symbol_multicall,
        get_vault_active_stake_multicall, get_vault_burner_multicall,
        get_vault_collateral_multicall, get_vault_current_epoch_multicall,
        get_vault_current_epoch_start_multicall, get_vault_delegator_multicall,
        get_vault_deposit_limit_multicall, get_vault_deposit_whitelist_multicall,
        get_vault_entity_multicall, get_vault_epoch_duration_multicall,
        get_vault_next_epoch_start_multicall, get_vault_slasher_multicall,
        get_vault_total_entities, get_vault_total_stake_multicall, get_vault_version_multicall,
        is_delegator, is_opted_in_vault, is_slasher, is_vault,
    },
    consts::get_vault_factory,
    contracts::{
        delegator::{
            base_delegator::IBaseDelegator, full_restake_delegator::IFullRestakeDelegator,
            network_restake_delegator::INetworkRestakeDelegator,
            operator_network_specific_delegator::IOperatorNetworkSpecificDelegator,
            operator_specific_delegator::IOperatorSpecificDelegator,
        },
        slasher::{base_slasher::IBaseSlasher, slasher::ISlasher, veto_slasher::IVetoSlasher},
        vault_configurator::IVaultConfigurator,
        IVault,
    },
    network_utils::NetworkInfo,
    utils::{get_network_link, get_subnetwork, get_vault_link},
    DelegatorType,
};

const SYMBIOTIC_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/symbioticfi/metadata-mainnet/refs/heads/main/vaults";
const VAULT_FILE_NAME: &str = "info.json";

#[derive(Debug)]
pub enum RowPrefix {
    Default,
    New,
    Old,
}

impl RowPrefix {
    pub fn row_name(self, name: &str) -> String {
        match self {
            RowPrefix::Default => return name.to_string(),
            RowPrefix::New => return format!("New {}", name),
            RowPrefix::Old => return format!("Old {}", name),
        }
    }
}

pub struct VaultDataTableBuilder {
    data: VaultData,
    table: Table,
}

impl VaultDataTableBuilder {
    pub fn from_vault_data(data: VaultData) -> Self {
        Self {
            data,
            table: Table::new(),
        }
    }

    pub fn with_table(mut self, table: Table) -> Self {
        self.table = table;
        self
    }

    pub fn with_name(mut self) -> Self {
        let name = self
            .data
            .symbiotic_metadata
            .clone()
            .map(|v| v.name)
            .unwrap_or("Unverified vault".to_string());
        self.table.add_row(row![
            Fcb -> "Name",
            get_vault_link(self.data.address, name)
        ]);
        self
    }

    pub fn with_network(
        mut self,
        network_address: Address,
        network_metadata: Option<NetworkInfo>,
    ) -> Self {
        let network_link = get_network_link(
            network_address,
            network_metadata
                .map(|v| v.name)
                .unwrap_or("UNVERIFIED".to_string()),
        );
        self.table.add_row(row![Fcb -> "Network", network_link]);
        self
    }

    pub fn with_subnetwork_identifier(
        mut self,
        network_address: Address,
        subnetwork: U96,
    ) -> eyre::Result<Self> {
        self.table.add_row(row![Fcb -> "Subnetwork Identifier",
            get_subnetwork(network_address, subnetwork)?.to_string()
        ]);
        Ok(self)
    }

    pub fn with_address(mut self) -> Self {
        let link = get_etherscan_address_link(self.data.address, self.data.address.to_string());
        self.table.add_row(row![Fcb ->"Address",  link]);
        self
    }

    pub fn with_version(mut self) -> Self {
        self.table
            .add_row(row![Fcb -> "Version",  self.data.version.clone().unwrap()]);
        self
    }

    pub fn with_collateral(mut self) -> Self {
        let txt = format!(
            "{} ({})",
            self.data.symbol.clone().unwrap(),
            self.data.collateral.clone().unwrap().to_string()
        );
        let link = get_etherscan_address_link(self.data.collateral.unwrap(), txt);
        self.table.add_row(row![Fcb -> "Collateral",  link]);
        self
    }

    pub fn with_delegator(mut self) -> Self {
        let link = get_etherscan_address_link(
            self.data.delegator.clone().unwrap(),
            self.data.delegator.clone().unwrap().to_string(),
        );
        self.table.add_row(row![Fcb -> "Delegator",  link]);
        self
    }

    pub fn with_slasher(mut self) -> Self {
        let link = get_etherscan_address_link(
            self.data.slasher.clone().unwrap(),
            self.data.slasher.clone().unwrap().to_string(),
        );
        self.table.add_row(row![Fcb -> "Slasher",  link]);
        self
    }

    pub fn with_burner(mut self) -> Self {
        let link = get_etherscan_address_link(
            self.data.burner.clone().unwrap(),
            self.data.burner.clone().unwrap().to_string(),
        );
        self.table.add_row(row![Fcb -> "Burner",  link]);
        self
    }

    pub fn with_deposit_limit(mut self) -> Self {
        let mut deposit_limit = self.data.deposit_limit_formatted().unwrap();
        if deposit_limit == "0.000" {
            deposit_limit = "-".to_string();
        }
        self.table
            .add_row(row![Fcb -> "Deposit limit",  deposit_limit]);

        self
    }

    pub fn with_deposit_whitelist(mut self) -> Self {
        let deposit_whitelist = match self.data.deposit_whitelist.unwrap() {
            true => "✅",
            false => "❌",
        };
        self.table
            .add_row(row![Fcb -> "Deposit whitelist",  deposit_whitelist]);
        self
    }

    pub fn with_total_stake(mut self) -> Self {
        self.table
            .add_row(row![Fcb -> "Total stake",  self.data.total_stake_formatted().unwrap()]);
        self
    }

    pub fn with_active_stake(mut self) -> Self {
        self.table.add_row(row![
            Fcb -> "Active stake",
            self.data.active_stake_formatted_with_percentage().unwrap()
        ]);
        self
    }

    pub fn with_current_epoch(mut self) -> Self {
        self.table
            .add_row(row![Fcb -> "Current epoch",  self.data.current_epoch.clone().unwrap()]);
        self
    }

    pub fn with_current_epoch_start(mut self) -> Self {
        self.table.add_row(row![
            Fcb -> "Current epoch start",
            parse_epoch_ts(self.data.current_epoch_start.clone().unwrap())
        ]);
        self
    }

    pub fn with_epoch_duration(mut self) -> Self {
        self.table.add_row(row![
            Fcb -> "Epoch duration",
            parse_duration_secs(self.data.epoch_duration.clone().unwrap())
        ]);
        self
    }

    pub fn with_next_epoch_start(mut self) -> Self {
        self.table.add_row(row![
            Fcb -> "Next epoch start",
            parse_epoch_ts(self.data.next_epoch_start.clone().unwrap())
        ]);
        self
    }

    pub fn with_time_till_next_epoch(mut self) -> Self {
        let now = Utc::now();
        let next_epoch_start = DateTime::<Utc>::from_timestamp(
            self.data.next_epoch_start.clone().unwrap().to::<i64>(),
            0,
        )
        .unwrap();

        let time_till_next_epoch = next_epoch_start.signed_duration_since(now);
        let time_till_next_epoch_str = parse_ts(time_till_next_epoch.num_seconds());

        self.table.add_row(row![
            Fcb -> "Time till next epoch",
            time_till_next_epoch_str
        ]);
        self
    }

    pub fn with_max_network_limit(
        mut self,
        max_network_limit: U256,
        row_prefix: RowPrefix,
    ) -> eyre::Result<Self> {
        let mut max_network_limit_formatted =
            format_number_with_decimals(max_network_limit, self.data.decimals.clone().unwrap())?;
        if max_network_limit_formatted == "0.000" {
            max_network_limit_formatted = "-".to_string();
        }
        self.table.add_row(row![Fcb -> row_prefix.row_name("Max Network Limit"), format!("{} ({} {})", max_network_limit.to_string(), max_network_limit_formatted, self.data.symbol.clone().unwrap())]);
        Ok(self)
    }

    pub fn with_network_limit(
        mut self,
        network_limit: U256,
        row_prefix: RowPrefix,
    ) -> eyre::Result<Self> {
        let mut network_limit_formatted =
            format_number_with_decimals(network_limit, self.data.decimals.clone().unwrap())?;
        if network_limit_formatted == "0.000" {
            network_limit_formatted = "-".to_string();
        }
        self.table.add_row(row![Fcb -> row_prefix.row_name("Network Limit"), format!("{} ({} {})", network_limit.to_string(), network_limit_formatted, self.data.symbol.clone().unwrap())]);
        Ok(self)
    }

    pub fn with_operator_network_limit(
        mut self,
        operator_network_limit: U256,
        row_prefix: RowPrefix,
    ) -> eyre::Result<Self> {
        let mut operator_network_limit_formatted = format_number_with_decimals(
            operator_network_limit,
            self.data.decimals.clone().unwrap(),
        )?;
        if operator_network_limit_formatted == "0.000" {
            operator_network_limit_formatted = "-".to_string();
        }
        self.table.add_row(row![Fcb -> row_prefix.row_name("Operator Network Limit"), format!("{} ({} {})", operator_network_limit.to_string(), operator_network_limit_formatted, self.data.symbol.clone().unwrap())]);
        Ok(self)
    }

    pub fn with_operator_network_shares(
        mut self,
        operator_network_shares: U256,
        row_prefix: RowPrefix,
    ) -> eyre::Result<Self> {
        let mut operator_network_shares_formatted = format_number_with_decimals(
            operator_network_shares,
            self.data.decimals.clone().unwrap(),
        )?;
        if operator_network_shares_formatted == "0.000" {
            operator_network_shares_formatted = "-".to_string();
        }
        self.table.add_row(row![Fcb -> row_prefix.row_name("Operator Network Shares"), format!("{} ({} {})", operator_network_shares.to_string(), operator_network_shares_formatted, self.data.symbol.clone().unwrap())]);
        Ok(self)
    }

    pub fn with_operator_stake(mut self, operator_stake: U256) -> eyre::Result<Self> {
        let mut operator_stake_formatted =
            format_number_with_decimals(operator_stake, self.data.decimals.clone().unwrap())?;
        if operator_stake_formatted == "0.000" {
            operator_stake_formatted = "-".to_string();
        }
        self.table.add_row(row![Fcb -> "Operator Stake", format!("{} ({} {})", operator_stake.to_string(), operator_stake_formatted, self.data.symbol.clone().unwrap())]);
        Ok(self)
    }

    pub fn with_total_operator_network_shares(
        mut self,
        total_operator_network_shares: U256,
    ) -> eyre::Result<Self> {
        let mut total_operator_network_shares_formatted = format_number_with_decimals(
            total_operator_network_shares,
            self.data.decimals.clone().unwrap(),
        )?;
        if total_operator_network_shares_formatted == "0.000" {
            total_operator_network_shares_formatted = "-".to_string();
        }
        self.table.add_row(row![Fcb -> "Total Operator Network Shares", format!("{} ({} {})", total_operator_network_shares.to_string(), total_operator_network_shares_formatted, self.data.symbol.clone().unwrap())]);
        Ok(self)
    }

    pub async fn with_all(self) -> eyre::Result<Self> {
        Ok(self
            .with_name()
            .with_address()
            .with_version()
            .with_collateral()
            .with_delegator()
            .with_slasher()
            .with_burner()
            .with_deposit_limit()
            .with_deposit_whitelist()
            .with_total_stake()
            .with_active_stake()
            .with_current_epoch()
            .with_current_epoch_start()
            .with_epoch_duration()
            .with_next_epoch_start())
    }

    pub fn build(self) -> Table {
        self.table
    }
}

#[derive(Clone)]
pub struct VaultData {
    pub address: Address,
    pub version: Option<u64>,
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
    pub symbiotic_metadata: Option<VaultInfo>,
}

impl VaultData {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            version: None,
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
            symbiotic_metadata: None,
        }
    }

    pub async fn load(
        chain_id: u64,
        address: Address,
        include_extra_metadata: bool,
        provider: &RetryProvider,
    ) -> eyre::Result<Self> {
        let mut vaults = fetch_vault_datas(provider, chain_id, vec![address]).await?;
        if include_extra_metadata {
            vaults = fetch_vault_extra_metadata(provider, chain_id, vaults).await?;
        }
        let vaults = fetch_token_datas(provider, chain_id, vaults).await?;
        let vault = vaults
            .first()
            .ok_or(eyre::eyre!("Vault not found"))?
            .clone()
            .with_symbiotic_metadata()
            .await?;
        Ok(vault)
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

    pub fn set_version(&mut self, version: U256) {
        self.version = Some(version.to::<u64>());
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

    pub fn set_symbiotic_metadata(&mut self, symbiotic_metadata: VaultInfo) {
        self.symbiotic_metadata = Some(symbiotic_metadata);
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
        let url = format!("{SYMBIOTIC_GITHUB_URL}/{}/{VAULT_FILE_NAME}", self.address);
        let res = reqwest::get(&url).await?;
        let vault_info: Option<VaultInfo> = serde_json::from_str(&res.text().await?).ok();
        self.symbiotic_metadata = vault_info;
        Ok(self)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct VaultInfo {
    pub name: String,
}

#[derive(Clone)]
pub struct TokenData {
    pub address: Address,
    pub decimals: u8,
    pub symbol: String,
}

pub fn get_encoded_vault_configurator_params(
    version: u64,
    collateral: Address,
    burner: Option<Address>,
    epoch_duration: U48,
    deposit_whitelist: bool,
    is_deposit_limit: bool,
    deposit_limit: U256,
    delegator_type: DelegatorType,
    delegator_hook: Option<Address>,
    with_slasher: bool,
    slasher_index: u64,
    veto_duration: U48,
    resolver_set_epochs_delay: U256,
    vault_admin_config: &AliasConfig,
) -> eyre::Result<String> {
    let network_limit_set_role_holders = vec![vault_admin_config.address];
    let operator_network_shares_set_role_holders = vec![vault_admin_config.address];

    let burner = burner.unwrap_or(Address::ZERO);
    let vault_params = IVault::InitParams {
        collateral,
        burner,
        epochDuration: epoch_duration,
        depositWhitelist: deposit_whitelist,
        isDepositLimit: is_deposit_limit,
        depositLimit: deposit_limit,
        defaultAdminRoleHolder: vault_admin_config.address,
        depositWhitelistSetRoleHolder: vault_admin_config.address,
        depositorWhitelistRoleHolder: vault_admin_config.address,
        isDepositLimitSetRoleHolder: vault_admin_config.address,
        depositLimitSetRoleHolder: vault_admin_config.address,
    };
    let delegator_params: Vec<u8> = match delegator_type {
        // NetworkRestakeDelegator (type 0)
        DelegatorType::NetworkRestakeDelegator => INetworkRestakeDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            networkLimitSetRoleHolders: network_limit_set_role_holders,
            operatorNetworkSharesSetRoleHolders: operator_network_shares_set_role_holders,
        }
        .abi_encode(),

        // FullRestakeDelegator (type 1)
        DelegatorType::FullRestakeDelegator => IFullRestakeDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            networkLimitSetRoleHolders: network_limit_set_role_holders,
            operatorNetworkLimitSetRoleHolders: operator_network_shares_set_role_holders,
        }
        .abi_encode(),

        // OperatorSpecificDelegator (type 2)
        DelegatorType::OperatorSpecificDelegator => IOperatorSpecificDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            networkLimitSetRoleHolders: network_limit_set_role_holders,
            operator: Address::ZERO,
        }
        .abi_encode(),

        // OperatorNetworkSpecificDelegator (type 3)
        DelegatorType::OperatorNetworkSpecificDelegator => {
            IOperatorNetworkSpecificDelegator::InitParams {
                baseParams: IBaseDelegator::BaseParams {
                    defaultAdminRoleHolder: vault_admin_config.address,
                    hook: delegator_hook.unwrap_or(Address::ZERO),
                    hookSetRoleHolder: vault_admin_config.address,
                },
                network: Address::ZERO,
                operator: Address::ZERO,
            }
            .abi_encode()
        }
    };

    let mut slasher_params = vec![];
    if with_slasher {
        slasher_params = match slasher_index {
            // Slasher (type 0)
            0 => ISlasher::InitParams {
                baseParams: IBaseSlasher::BaseParams {
                    isBurnerHook: !burner.is_zero(),
                },
            }
            .abi_encode(),

            // VetoSlasher (type 1)
            1 => IVetoSlasher::InitParams {
                baseParams: IBaseSlasher::BaseParams {
                    isBurnerHook: !burner.is_zero(),
                },
                vetoDuration: veto_duration,
                resolverSetEpochsDelay: resolver_set_epochs_delay,
            }
            .abi_encode(),
            _ => {
                print_error_message("Invalid slasher index.");
                return Err(eyre::eyre!(""));
            }
        };
    }

    let configurator_init_params = IVaultConfigurator::InitParams {
        version,
        owner: vault_admin_config.address,
        vaultParams: vault_params.abi_encode().into(),
        delegatorIndex: delegator_type as u64,
        delegatorParams: delegator_params.into(),
        withSlasher: with_slasher,
        slasherIndex: slasher_index,
        slasherParams: slasher_params.into(),
    };

    let configurator_params_bytes: Bytes = configurator_init_params.abi_encode_params().into();
    let encoded = format!("0x{}", configurator_params_bytes.encode_hex());
    Ok(encoded)
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
        get_vault_version_multicall(&mut multicall, *vault, true);
    }

    let vault_datas = multicall.call().await?.into_iter().chunks(5);

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
        let version = vault_data[4]
            .as_ref()
            .map(|data| data.as_uint())
            .ok()
            .flatten();

        // Skip if any of the vault data is missing/errored
        if collateral.is_none()
            || delegator.is_none()
            || total_stake.is_none()
            || active_stake.is_none()
            || version.is_none()
        {
            println!("{} {}", "Skipping vault: ".bright_yellow(), vault);
            continue;
        }

        let collateral = collateral.unwrap();
        let delegator = delegator.unwrap();
        let total_stake = total_stake.unwrap();
        let active_stake = active_stake.unwrap();
        let version = version.unwrap();

        let mut vault_data = VaultData::new(vault);
        vault_data.set_collateral(collateral);
        vault_data.set_delegator(delegator);
        vault_data.set_total_stake(total_stake.0);
        vault_data.set_active_stake(active_stake.0);
        vault_data.set_version(version.0);

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

/// Fetches token metadata (decimals and symbol) for a vault collateral token
///
/// # Arguments
///
/// * `provider` - The Ethereum provider to use for making RPC calls
/// * `chain_id` - The chain ID to use for the multicall contract
/// * `vault` - Address of the vault to fetch token data for
///
/// # Returns
///
/// Returns a `TokenData` struct containing the fetched token metadata
pub async fn fetch_token_data(
    chain_id: u64,
    token: Address,
    provider: &RetryProvider,
) -> eyre::Result<Option<TokenData>> {
    let mut multicall = Multicall::with_chain_id(&provider, chain_id)?;
    multicall.set_version(MulticallVersion::Multicall3);

    get_token_decimals_multicall(&mut multicall, token, false);
    get_token_symbol_multicall(&mut multicall, token, false);

    let token_call = multicall.call().await?;
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
        return Ok(None);
    }

    let token = TokenData {
        address: token,
        decimals: decimals.unwrap().0.try_into()?,
        symbol: symbol.unwrap().to_string(),
    };

    Ok(Some(token))
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

pub async fn fetch_vault_symbiotic_metadata(
    vaults: Vec<VaultData>,
) -> eyre::Result<Vec<VaultData>> {
    let mut out = Vec::with_capacity(vaults.len());
    for vault in vaults {
        out.push(vault.with_symbiotic_metadata().await?);
    }

    Ok(out)
}

pub async fn get_vault_network_limit_formatted<A: TryInto<Address>>(
    provider: &RetryProvider,
    network: A,
    subnetwork: U96,
    vault: &VaultData,
    delegator: A,
    delegator_type: DelegatorType,
    max_network_limit: String,
) -> eyre::Result<String>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault_limit_display = if delegator_type == DelegatorType::OperatorNetworkSpecificDelegator {
        max_network_limit
    } else {
        let network_limit = get_network_limit(network, subnetwork, delegator, provider).await?;
        let network_limit_formatted =
            format_number_with_decimals(network_limit, vault.decimals.clone().unwrap())?;
        if network_limit_formatted == "0.000" {
            format!(
                "{} (- {})",
                network_limit.to_string(),
                vault.symbol.clone().unwrap()
            )
        } else {
            format!(
                "{} ({} {})",
                network_limit.to_string(),
                network_limit_formatted,
                vault.symbol.clone().unwrap()
            )
        }
    };
    Ok(vault_limit_display)
}

pub async fn validate_vault_status<A: TryInto<Address>>(
    vault: A,
    vault_factory: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_vault = print_loading_until_async(
        "Validating vault status",
        is_vault(vault, vault_factory, provider),
    )
    .await?;

    if !is_vault {
        eyre::bail!("Provided address is not a valid Symbiotic vault.");
    }

    Ok(())
}

pub async fn validate_delegator_status<A: TryInto<Address>>(
    delegator: A,
    delegator_factory: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_delegator = print_loading_until_async(
        "Checking delegator status",
        is_delegator(delegator, delegator_factory, &provider),
    )
    .await?;

    if !is_delegator {
        eyre::bail!("Provided address is not a valid Symbiotic delegator.");
    }

    Ok(())
}

pub async fn validate_slasher_status<A: TryInto<Address>>(
    slasher: A,
    slasher_factory: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_slasher = print_loading_until_async(
        "Checking slasher status",
        is_slasher(slasher, slasher_factory, &provider),
    )
    .await?;

    if !is_slasher {
        eyre::bail!("Provided address is not a valid Symbiotic slasher.");
    }

    Ok(())
}

pub async fn validate_operator_vault_opt_in_status<A: TryInto<Address>>(
    operator: A,
    vault: A,
    opt_in_service: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_opted_in = print_loading_until_async(
        "Checking opted in status",
        is_opted_in_vault(operator, vault, opt_in_service, &provider),
    )
    .await?;

    if !is_opted_in {
        eyre::bail!("Operator is not opted in vault.");
    }

    Ok(())
}
