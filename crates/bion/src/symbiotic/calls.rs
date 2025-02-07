//! A collection of function that call important information from the Symbiotic contracts.

use alloy_dyn_abi::DynSolValue;
use alloy_network::{Network, TransactionBuilder};
use alloy_primitives::{aliases::U48, Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::{JsonAbiExt, SolCall};
use alloy_transport::Transport;
use cast::Cast;
use eyre::Result;
use foundry_common::provider::RetryProvider;
use multicall::Multicall;

use std::str::FromStr;

use crate::symbiotic::contracts::vault_factory::VaultFactory;

use super::contracts::{
    delegator_factory::DelegatorFactory,
    erc20,
    slasher_factory::SlasherFactory,
    INetworkRegistry, IOperatorRegistry,
    IOptInService::{self, isOptedInCall, isOptedInReturn},
    IVault::{self, totalStakeCall, totalStakeReturn},
};

pub async fn get_token_decimals<A: TryInto<Address>>(
    token: A,
    provider: &RetryProvider,
) -> Result<u8>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let token = token.try_into()?;

    let call = erc20::decimalsCall::new(());

    let erc20::decimalsReturn { _0: decimals } = call_and_decode(call, token, provider).await?;

    Ok(decimals)
}

pub async fn get_token_symbol<A: TryInto<Address>>(
    token: A,
    provider: &RetryProvider,
) -> Result<String>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let token = token.try_into()?;

    let call = erc20::symbolCall::new(());

    let erc20::symbolReturn { _0: symbol } = call_and_decode(call, token, provider).await?;

    Ok(symbol)
}

pub async fn get_vault_active_stake<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;
    let vault_contract = vault.try_into()?;

    let call = totalStakeCall {};

    let totalStakeReturn { _0: active_stake } =
        call_and_decode(call, vault_contract, provider).await?;

    Ok(active_stake)
}

pub async fn get_vault_burner<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<Address>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::burnerCall::new(());

    let IVault::burnerReturn { _0: burner } = call_and_decode(call, vault, provider).await?;

    Ok(burner)
}

pub fn get_vault_burner_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("burner").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_collateral<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<Address>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::collateralCall::new(());

    let IVault::collateralReturn { _0: collateral } =
        call_and_decode(call, vault, provider).await?;

    Ok(collateral)
}

pub fn get_vault_collateral_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("collateral").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_current_epoch<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::currentEpochCall::new(());

    let IVault::currentEpochReturn { _0: current_epoch } =
        call_and_decode(call, vault, provider).await?;

    Ok(current_epoch)
}

pub fn get_vault_current_epoch_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("currentEpoch").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_current_epoch_start<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U48>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::currentEpochStartCall::new(());

    let IVault::currentEpochStartReturn { _0: current_epoch } =
        call_and_decode(call, vault, provider).await?;

    Ok(current_epoch)
}

pub fn get_vault_current_epoch_start_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("currentEpochStart").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_delegator<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<Address>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::delegatorCall::new(());

    let IVault::delegatorReturn { _0: delegator } = call_and_decode(call, vault, provider).await?;

    Ok(delegator)
}

pub fn get_vault_delegator_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("delegator").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_deposit_limit<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::depositLimitCall::new(());

    let IVault::depositLimitReturn { _0: deposit_limit } =
        call_and_decode(call, vault, provider).await?;

    Ok(deposit_limit)
}

pub fn get_vault_deposit_limit_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("depositLimit").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_deposit_whitelist<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::depositWhitelistCall::new(());

    let IVault::depositWhitelistReturn {
        _0: deposit_whitelist,
    } = call_and_decode(call, vault, provider).await?;

    Ok(deposit_whitelist)
}

pub fn get_vault_deposit_whitelist_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("depositWhitelist").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_epoch_duration<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U48>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::epochDurationCall::new(());

    let IVault::epochDurationReturn { _0: epoch_duration } =
        call_and_decode(call, vault, provider).await?;

    Ok(epoch_duration)
}

pub fn get_vault_epoch_duration_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("epochDuration").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_entity<A: TryInto<Address>>(
    vault_factory: A,
    index: U256,
    provider: &RetryProvider,
) -> Result<Address>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let factory = vault_factory.try_into()?;

    let call = VaultFactory::entityCall::new((index,));

    let VaultFactory::entityReturn { _0: entity } =
        call_and_decode(call, factory, provider).await?;

    Ok(entity)
}

pub fn get_vault_entity_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault_factory: Address,
    index: U256,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = VaultFactory::abi::functions();
    // can safely unwrap
    let function = abi.get("entity").unwrap().first().unwrap();

    multicall.add_call(
        vault_factory,
        function,
        &[DynSolValue::from(index)],
        allow_failure,
    )
}

pub async fn get_vault_next_epoch_start<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U48>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::nextEpochStartCall::new(());

    let IVault::nextEpochStartReturn {
        _0: next_epoch_start,
    } = call_and_decode(call, vault, provider).await?;

    Ok(next_epoch_start)
}

pub fn get_vault_next_epoch_start_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("nextEpochStart").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_slasher<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<Address>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;

    let call = IVault::slasherCall::new(());

    let IVault::slasherReturn { _0: slasher } = call_and_decode(call, vault, provider).await?;

    Ok(slasher)
}

pub fn get_vault_slasher_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    // can safely unwrap
    let function = abi.get("slasher").unwrap().first().unwrap();

    multicall.add_call(vault, function, &[], allow_failure)
}

pub async fn get_vault_total_entities<A: TryInto<Address>>(
    vault_factory: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let factory = vault_factory.try_into()?;

    let call = VaultFactory::totalEntitiesCall::new(());

    let VaultFactory::totalEntitiesReturn { _0: total_entities } =
        call_and_decode(call, factory, provider).await?;

    Ok(total_entities)
}

pub async fn get_vault_total_stake<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault: Address = vault.try_into()?;
    let vault_contract: Address = vault.try_into()?;

    let call = totalStakeCall {};

    let totalStakeReturn { _0: total_stake } =
        call_and_decode(call, vault_contract, provider).await?;

    Ok(total_stake)
}

pub async fn is_delegator<A: TryInto<Address>>(
    delegator: A,
    delegator_factory: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let delegator = delegator.try_into()?;
    let delegator_factory = delegator_factory.try_into()?;

    let call = DelegatorFactory::isEntityCall::new((delegator,));

    let DelegatorFactory::isEntityReturn { _0: is_entity } =
        call_and_decode(call, delegator_factory, provider).await?;

    Ok(is_entity)
}

/// Checks if an operator is registered in Symbiotic
///
/// # Arguments
///
/// * `address` - The operator's address to check
/// * `operator_registry` - The address of the operator registry contract
/// * `provider` - The provider used to make the contract call
///
/// # Returns
///
/// * `Result<bool, eyre::Error>` - Returns true if operator is registered in Symbiotic, false otherwise
///
/// # Errors
///
/// Returns an error if:
/// * Any of the addresses fail to convert
/// * The contract call fails
pub async fn is_operator<A: TryInto<Address>>(
    address: A,
    operator_registry: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let address: Address = address.try_into()?;
    let operator_registry: Address = operator_registry.try_into()?;

    let call = IOperatorRegistry::isEntityCall { account: address };

    let IOperatorRegistry::isEntityReturn { _0: is_entity } =
        call_and_decode(call, operator_registry, provider).await?;

    Ok(is_entity)
}

/// Gets the opt-in status of an operator for a specific network from the opt-in service contract
///
/// # Arguments
///
/// * `address` - The operator's address to check
/// * `network` - The network address to check opt-in status for
/// * `opt_in_service` - The address of the opt-in service contract
/// * `provider` - The provider used to make the contract call
///
/// # Returns
///
/// * `Result<bool, eyre::Error>` - Returns true if operator is opted in to the network, false otherwise
///
/// # Errors
///
/// Returns an error if:
/// * Any of the addresses fail to convert
/// * The contract call fails
pub async fn is_opted_in_network<A: TryInto<Address>>(
    address: A,
    network: A,
    opt_in_service: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let address: Address = address.try_into()?;
    let network: Address = network.try_into()?;
    let opt_in_service: Address = opt_in_service.try_into()?;

    let call = isOptedInCall::new((address, network));

    let isOptedInReturn { _0: is_opted_in } =
        call_and_decode(call, opt_in_service, provider).await?;

    Ok(is_opted_in)
}

pub async fn is_opted_in_vault<A: TryInto<Address>>(
    address: A,
    vault: A,
    opt_in_service: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let address: Address = address.try_into()?;
    let vault: Address = vault.try_into()?;
    let opt_in_service: Address = opt_in_service.try_into()?;

    let call = isOptedInCall::new((address, vault));

    let isOptedInReturn { _0: is_opted_in } =
        call_and_decode(call, opt_in_service, provider).await?;

    Ok(is_opted_in)
}

pub async fn is_network<A: TryInto<Address>>(
    network: A,
    network_registry: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let network = network.try_into()?;
    let network_registry = network_registry.try_into()?;

    let call = INetworkRegistry::isEntityCall { account: network };

    let INetworkRegistry::isEntityReturn { _0: is_entity } =
        call_and_decode(call, network_registry, provider).await?;

    Ok(is_entity)
}

pub async fn is_slasher<A: TryInto<Address>>(
    slasher: A,
    slasher_factory: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let slasher = slasher.try_into()?;
    let slasher_factory = slasher_factory.try_into()?;

    let call = SlasherFactory::isEntityCall::new((slasher,));

    let SlasherFactory::isEntityReturn { _0: is_entity } =
        call_and_decode(call, slasher_factory, provider).await?;

    Ok(is_entity)
}

pub async fn is_vault<A: TryInto<Address>>(
    vault: A,
    vault_factory: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault = vault.try_into()?;
    let vault_factory = vault_factory.try_into()?;

    let call = VaultFactory::isEntityCall::new((vault,));

    let VaultFactory::isEntityReturn { _0: is_entity } =
        call_and_decode(call, vault_factory, provider).await?;

    Ok(is_entity)
}

/// Private function to make a contract call and decode the response
async fn call_and_decode<C: SolCall>(
    call: C,
    to: Address,
    provider: &RetryProvider,
) -> Result<C::Return> {
    let call_data: Vec<u8> = call.abi_encode();

    let mut req = TransactionRequest::default().to(to);
    req.set_input(call_data);

    let req = WithOtherFields::new(req);

    let cast = Cast::new(provider);
    let data = cast.call(&req, None, None).await?;
    let data = Bytes::from_str(data.as_str())?;
    let data = C::abi_decode_returns(data.as_ref(), true)?;

    Ok(data)
}

/// Multicall variant of get_token_decimals
pub fn get_token_decimals_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    token: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let function = erc20::decimalsCall::abi();
    multicall.add_call(token, &function, &[], allow_failure)
}

/// Multicall variant of get_token_symbol
pub fn get_token_symbol_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    token: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let function = erc20::symbolCall::abi();
    multicall.add_call(token, &function, &[], allow_failure)
}

/// Multicall variant of get_vault_active_stake
pub fn get_vault_active_stake_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    let function = abi.get("totalStake").unwrap().first().unwrap();
    multicall.add_call(vault, function, &[], allow_failure)
}

/// Multicall variant of get_vault_total_entities
pub fn get_vault_total_entities_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault_factory: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = VaultFactory::abi::functions();
    let function = abi.get("totalEntities").unwrap().first().unwrap();
    multicall.add_call(vault_factory, function, &[], allow_failure)
}

/// Multicall variant of get_vault_total_stake
pub fn get_vault_total_stake_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IVault::abi::functions();
    let function = abi.get("totalStake").unwrap().first().unwrap();
    multicall.add_call(vault, function, &[], allow_failure)
}

/// Multicall variant of is_operator
pub fn is_operator_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    address: Address,
    operator_registry: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IOperatorRegistry::abi::functions();
    let function = abi.get("isEntity").unwrap().first().unwrap();
    multicall.add_call(
        operator_registry,
        function,
        &[DynSolValue::from(address)],
        allow_failure,
    )
}

/// Multicall variant of is_opted_in_network
pub fn is_opted_in_network_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    address: Address,
    network: Address,
    opt_in_service: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IOptInService::abi::functions();
    let function = abi.get("isOptedIn").unwrap().first().unwrap();
    multicall.add_call(
        opt_in_service,
        function,
        &[DynSolValue::from(address), DynSolValue::from(network)],
        allow_failure,
    )
}

/// Multicall variant of is_opted_in_vault
pub fn is_opted_in_vault_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    address: Address,
    vault: Address,
    opt_in_service: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = IOptInService::abi::functions();
    let function = abi.get("isOptedIn").unwrap().first().unwrap();
    multicall.add_call(
        opt_in_service,
        function,
        &[DynSolValue::from(address), DynSolValue::from(vault)],
        allow_failure,
    )
}

/// Multicall variant of is_network
pub fn is_network_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    network: Address,
    network_registry: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = INetworkRegistry::abi::functions();
    let function = abi.get("isEntity").unwrap().first().unwrap();
    multicall.add_call(
        network_registry,
        function,
        &[DynSolValue::from(network)],
        allow_failure,
    )
}

/// Multicall variant of is_vault
pub fn is_vault_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    vault: Address,
    vault_factory: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = VaultFactory::abi::functions();
    let function = abi.get("isEntity").unwrap().first().unwrap();
    multicall.add_call(
        vault_factory,
        function,
        &[DynSolValue::from(vault)],
        allow_failure,
    )
}
