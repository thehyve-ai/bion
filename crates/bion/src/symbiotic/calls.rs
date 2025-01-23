//! A collection of function that call important information from the Symbiotic contracts.

use alloy_network::TransactionBuilder;
use alloy_primitives::Address;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::SolCall;
use cast::Cast;
use foundry_common::provider::RetryProvider;

use super::contracts::IOptInService::{isOptedInCall, isOptedInReturn};

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
    provider: RetryProvider,
) -> Result<bool, eyre::Error>
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
    provider: RetryProvider,
) -> Result<bool, eyre::Error>
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

/// Private function to make a contract call and decode the response
async fn call_and_decode<C: SolCall>(
    call: C,
    to: Address,
    provider: RetryProvider,
) -> Result<C::Return, eyre::Error> {
    let call_data: Vec<u8> = call.abi_encode();

    let mut req = TransactionRequest::default().to(to);
    req.set_input(call_data);

    let req = WithOtherFields::new(req);

    let cast = Cast::new(provider);
    let data = cast.call(&req, None, None).await?;
    let data = C::abi_decode_returns(data.as_ref(), true)?;

    Ok(data)
}
