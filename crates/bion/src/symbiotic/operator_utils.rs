use alloy_primitives::Address;
use foundry_common::provider::RetryProvider;

use crate::utils::print_loading_until_async;

use super::calls::is_operator;

#[derive(Clone, Debug)]
pub struct OperatorData {
    pub address: Address,
    pub symbiotic_metadata: Option<OperatorInfo>,
}

impl OperatorData {
    pub fn new(address: Address) -> Self {
        Self { address, symbiotic_metadata: None }
    }
}

#[derive(Clone, Debug)]
pub struct OperatorInfo {
    pub name: String,
}

pub async fn validate_operator_symbiotic_status<A: TryInto<Address>>(
    operator: A,
    operator_registry: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_operator = print_loading_until_async(
        "Validating operator Symbiotic status",
        is_operator(operator, operator_registry, provider),
    )
    .await?;

    if !is_operator {
        eyre::bail!("Operator is not registered Symbiotic.");
    }

    Ok(())
}
