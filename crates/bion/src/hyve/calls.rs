use alloy_dyn_abi::DynSolValue;
use alloy_network::{Network, TransactionBuilder};
use alloy_primitives::{aliases::U48, Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::SolCall;
use alloy_transport::Transport;
use cast::Cast;
use eyre::Result;
use foundry_common::provider::RetryProvider;
use multicall::Multicall;

use std::str::FromStr;

use super::contracts::hyve_network_middleware::HyveNetworkMiddleware;

pub async fn get_current_epoch<A: TryInto<Address>>(
    middleware: A,
    provider: &RetryProvider,
) -> Result<U48>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveNetworkMiddleware::getCurrentEpochCall::new(());

    let HyveNetworkMiddleware::getCurrentEpochReturn { _0: epoch } =
        call_and_decode(call, middleware, provider).await?;

    Ok(epoch)
}

// Decide if needed
pub fn get_current_epoch_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    middleware: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = HyveNetworkMiddleware::abi::functions();
    // can safely unwrap
    let function = abi.get("getCurrentEpoch").unwrap().first().unwrap();

    multicall.add_call(middleware, function, &[], allow_failure)
}

pub async fn get_epoch_start<A: TryInto<Address>>(
    epoch: U48,
    middleware: A,
    provider: &RetryProvider,
) -> Result<U48>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveNetworkMiddleware::getEpochStartCall::new((epoch,));

    let HyveNetworkMiddleware::getEpochStartReturn { _0: epoch_start } =
        call_and_decode(call, middleware, provider).await?;

    Ok(epoch_start)
}

// Decide if needed
pub fn get_epoch_start_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    epoch: U256,
    middleware: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = HyveNetworkMiddleware::abi::functions();
    // can safely unwrap
    let function = abi.get("getEpochStart").unwrap().first().unwrap();

    multicall.add_call(
        middleware,
        function,
        &[DynSolValue::from(epoch)],
        allow_failure,
    )
}

pub async fn key_was_active_at<A: TryInto<Address>>(
    timestamp: U48,
    key: Bytes,
    middleware: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveNetworkMiddleware::keyWasActiveAtCall::new((timestamp, key));

    let HyveNetworkMiddleware::keyWasActiveAtReturn { _0: is_active } =
        call_and_decode(call, middleware, provider).await?;

    Ok(is_active)
}

// Decide if needed
pub fn key_was_active_at_multicall<T, P, N>(
    multicall: &mut Multicall<T, P, N>,
    timestamp: U256,
    key: Bytes,
    middleware: Address,
    allow_failure: bool,
) -> usize
where
    N: Network,
    T: Transport + Clone,
    P: Provider<T, N> + Clone,
{
    let abi = HyveNetworkMiddleware::abi::functions();
    // can safely unwrap
    let function = abi.get("keyWasActiveAt").unwrap().first().unwrap();

    multicall.add_call(
        middleware,
        function,
        &[
            DynSolValue::from(timestamp),
            DynSolValue::from(key.as_ref().to_vec()),
        ],
        allow_failure,
    )
}

pub async fn operator_key<A: TryInto<Address>>(
    operator: A,
    middleware: A,
    provider: &RetryProvider,
) -> Result<Bytes>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let operator = operator.try_into()?;
    let middleware = middleware.try_into()?;

    let call = HyveNetworkMiddleware::operatorKeyCall::new((operator,));

    let HyveNetworkMiddleware::operatorKeyReturn { _0: key } =
        call_and_decode(call, middleware, provider).await?;

    Ok(key)
}

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
