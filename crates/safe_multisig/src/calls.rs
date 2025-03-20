use alloy_network::TransactionBuilder;
use alloy_primitives::{hex::ToHexExt, Address, Bytes, B256, U256};
use alloy_rpc_types::{serde_helpers::WithOtherFields, TransactionRequest};
use alloy_sol_types::SolCall;
use cast::Cast;
use foundry_common::provider::RetryProvider;

use std::str::FromStr;

use crate::{
    contracts::safe::Safe,
    transaction_data::{ExecutableSafeTransaction, SafeTransactionData},
};

pub fn exec_transaction<A: TryInto<Address>>(
    safe_tx: &SafeTransactionData,
    signature: &[u8],
    safe: A,
) -> eyre::Result<ExecutableSafeTransaction>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let safe: Address = safe.try_into()?;

    let call = Safe::execTransactionCall::new((
        Address::parse_checksummed(&safe_tx.to, None)?,
        safe_tx.value.try_into()?,
        safe_tx.data.clone(),
        safe_tx.operation,
        safe_tx.safe_tx_gas.try_into()?,
        safe_tx.base_gas.try_into()?,
        safe_tx.gas_price.try_into()?,
        safe_tx.gas_token,
        safe_tx.refund_receiver,
        Bytes::copy_from_slice(signature),
    ));

    let calldata = call.abi_encode().encode_hex_with_prefix();

    Ok(ExecutableSafeTransaction { safe_address: safe, input_data: calldata })
}

pub async fn get_nonce<A: TryInto<Address>>(safe: A, provider: &RetryProvider) -> eyre::Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let safe = safe.try_into()?;

    let call = Safe::nonceCall::new(());

    let Safe::nonceReturn { _0: nonce } = call_and_decode(call, safe, provider).await?;

    Ok(nonce)
}

pub async fn get_owners<A: TryInto<Address>>(
    safe: A,
    provider: &RetryProvider,
) -> eyre::Result<Vec<Address>>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let safe = safe.try_into()?;

    let call = Safe::getOwnersCall::new(());

    let Safe::getOwnersReturn { _0: owners } = call_and_decode(call, safe, provider).await?;

    Ok(owners)
}

pub async fn get_threshold<A: TryInto<Address>>(
    safe: A,
    provider: &RetryProvider,
) -> eyre::Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let safe = safe.try_into()?;

    let call = Safe::getThresholdCall::new(());

    let Safe::getThresholdReturn { _0: threshold } = call_and_decode(call, safe, provider).await?;

    Ok(threshold)
}

pub async fn get_transaction_hash<A: TryInto<Address>>(
    safe_tx: &SafeTransactionData,
    safe: A,
    provider: &RetryProvider,
) -> eyre::Result<B256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let safe = safe.try_into()?;

    let call = Safe::getTransactionHashCall::new((
        Address::parse_checksummed(&safe_tx.to, None)?,
        safe_tx.value.try_into()?,
        safe_tx.data.clone(),
        safe_tx.operation,
        safe_tx.safe_tx_gas.try_into()?,
        safe_tx.base_gas.try_into()?,
        safe_tx.gas_price.try_into()?,
        safe_tx.gas_token,
        safe_tx.refund_receiver,
        safe_tx.nonce.try_into()?,
    ));

    let Safe::getTransactionHashReturn { _0: tx_hash } =
        call_and_decode(call, safe, provider).await?;

    Ok(tx_hash)
}

pub async fn get_version<A: TryInto<Address>>(
    safe: A,
    provider: &RetryProvider,
) -> eyre::Result<String>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let safe = safe.try_into()?;

    let call = Safe::VERSIONCall::new(());

    let Safe::VERSIONReturn { _0: version } = call_and_decode(call, safe, provider).await?;

    Ok(version)
}

pub async fn is_owner<A: TryInto<Address>>(
    address: A,
    safe: A,
    provider: &RetryProvider,
) -> eyre::Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let address = address.try_into()?;
    let safe = safe.try_into()?;

    let call = Safe::isOwnerCall::new((address,));

    let Safe::isOwnerReturn { _0: is_owner } = call_and_decode(call, safe, provider).await?;

    Ok(is_owner)
}

/// Private function to make a contract call and decode the response
async fn call_and_decode<C: SolCall>(
    call: C,
    to: Address,
    provider: &RetryProvider,
) -> eyre::Result<C::Return> {
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
