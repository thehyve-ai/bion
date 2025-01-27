//! A collection of function that call important information from the Symbiotic contracts.

use std::{marker::PhantomData, str::FromStr};

use alloy_network::{AnyNetwork, TransactionBuilder};
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::SolCall;
use alloy_transport::Transport;
use cast::Cast;
use eyre::Result;
use foundry_common::provider::RetryProvider;

use crate::symbiotic::contracts::vault_factory::{self, VaultFactory};

use super::contracts::{
    // vault_factory::VaultFactory,
    INetworkRegistry,
    IOperatorRegistry,
    IOptInService::{isOptedInCall, isOptedInReturn},
    IVault::{totalStakeCall, totalStakeReturn},
};

pub async fn get_operator_registry_status<A: TryInto<Address>>(
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
pub async fn get_operator_network_opt_in_status<A: TryInto<Address>>(
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

pub async fn get_operator_vault_opt_in_status<A: TryInto<Address>>(
    address: A,
    vault: A,
    vault_opt_in_service: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let address: Address = address.try_into()?;
    let vault: Address = vault.try_into()?;
    let opt_in_service: Address = vault_opt_in_service.try_into()?;

    let call = isOptedInCall::new((address, vault));

    let isOptedInReturn { _0: is_opted_in } =
        call_and_decode(call, opt_in_service, provider).await?;

    Ok(is_opted_in)
}

pub async fn get_vault_active_stake<A: TryInto<Address>>(
    vault: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let vault: Address = vault.try_into()?;
    let vault_contract: Address = vault.try_into()?;

    let call = totalStakeCall {};

    let totalStakeReturn { _0: active_stake } =
        call_and_decode(call, vault_contract, provider).await?;

    Ok(active_stake)
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
