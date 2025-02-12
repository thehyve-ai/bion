use alloy_primitives::Address;
use foundry_common::provider::RetryProvider;

use crate::utils::print_loading_until_async;

use super::calls::is_operator;

pub async fn validate_operator_status<A: TryInto<Address>>(
    operator: A,
    operator_registry: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_vault = print_loading_until_async(
        "Validating operator status",
        is_operator(operator, operator_registry, provider),
    )
    .await?;

    if !is_vault {
        eyre::bail!("Operator is not registered Symbiotic.");
    }

    Ok(())
}
