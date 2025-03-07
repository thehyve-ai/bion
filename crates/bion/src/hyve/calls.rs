use alloy_network::TransactionBuilder;
use alloy_primitives::{aliases::U48, Address, Bytes, U256};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::SolCall;
use cast::Cast;
use eyre::Result;
use foundry_common::provider::RetryProvider;

use std::str::FromStr;

use super::contracts::{hyve_network_middleware::HyveNetworkMiddleware, hyve_reader::HyveReader};

pub async fn active_operators<A: TryInto<Address>>(
    middleware: A,
    provider: &RetryProvider,
) -> Result<Vec<Address>>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveReader::activeOperatorsCall::new(());

    let HyveReader::activeOperatorsReturn { _0: operators } =
        call_and_decode(call, middleware, provider).await?;

    Ok(operators)
}

pub async fn active_operators_at<A: TryInto<Address>>(
    timestamp: U48,
    middleware: A,
    provider: &RetryProvider,
) -> Result<Vec<Address>>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveReader::activeOperatorsAtCall::new((timestamp,));

    let HyveReader::activeOperatorsAtReturn { _0: operators } =
        call_and_decode(call, middleware, provider).await?;

    Ok(operators)
}

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

pub async fn is_operator_registered<A: TryInto<Address>>(
    operator: A,
    middleware: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let operator = operator.try_into()?;
    let middleware = middleware.try_into()?;

    let call = HyveReader::isOperatorRegisteredCall::new((operator,));

    let HyveReader::isOperatorRegisteredReturn { _0: is_registered } =
        call_and_decode(call, middleware, provider).await?;

    Ok(is_registered)
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

pub async fn operator_was_active_at<A: TryInto<Address>>(
    timestamp: U48,
    operator: A,
    middleware: A,
    provider: &RetryProvider,
) -> Result<bool>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let operator = operator.try_into()?;
    let middleware = middleware.try_into()?;

    let call = HyveReader::operatorWasActiveAtCall::new((timestamp, operator));

    let HyveReader::operatorWasActiveAtReturn { _0: is_active } =
        call_and_decode(call, middleware, provider).await?;

    Ok(is_active)
}

pub async fn operator_with_times_at<A: TryInto<Address>>(
    pos: U256,
    middleware: A,
    provider: &RetryProvider,
) -> Result<(Address, U48, U48)>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveReader::operatorWithTimesAtCall::new((pos,));

    let HyveReader::operatorWithTimesAtReturn {
        _0: operator,
        _1: start,
        _2: end,
    } = call_and_decode(call, middleware, provider).await?;

    Ok((operator, start, end))
}

pub async fn operators_length<A: TryInto<Address>>(
    middleware: A,
    provider: &RetryProvider,
) -> Result<U256>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveReader::operatorsLengthCall::new(());

    let HyveReader::operatorsLengthReturn { _0: length } =
        call_and_decode(call, middleware, provider).await?;

    Ok(length)
}

pub async fn slashing_window<A: TryInto<Address>>(
    middleware: A,
    provider: &RetryProvider,
) -> Result<U48>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let middleware = middleware.try_into()?;

    let call = HyveReader::SLASHING_WINDOWCall::new(());

    let HyveReader::SLASHING_WINDOWReturn { _0: window } =
        call_and_decode(call, middleware, provider).await?;

    Ok(window)
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
