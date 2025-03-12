use alloy_primitives::{aliases::U48, Address};
use foundry_common::provider::RetryProvider;

use crate::utils::print_loading_until_async;

use super::calls::{
    get_current_epoch, get_epoch_start, is_operator_registered, operator_was_active_at,
};

#[allow(dead_code)]
pub async fn validate_operator_hyve_middleware_status<A: TryInto<Address>>(
    operator: A,
    middleware: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_registred = print_loading_until_async(
        "Validating operator HyveDA status",
        is_operator_registered(operator, middleware, provider),
    )
    .await?;

    if !is_registred {
        eyre::bail!("Operator is not registered the HyveDA middleware.");
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn is_operator_active(
    operator: Address,
    middleware: Address,
    reader: Address,
    provider: &RetryProvider,
) -> eyre::Result<bool> {
    let current_epoch = get_current_epoch(middleware, provider).await?;
    let next_epoch = current_epoch + U48::from(1);
    let epoch_timestamp = get_epoch_start(next_epoch, middleware, &provider).await?;
    let is_operator_active =
        operator_was_active_at(epoch_timestamp, operator, reader, &provider).await?;

    Ok(is_operator_active)
}
