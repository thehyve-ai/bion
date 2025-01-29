use alloy_primitives::{utils::format_units, U256};
use cast::Cast;
use foundry_common::provider::RetryProvider;

pub async fn get_chain_id(provider: &RetryProvider) -> eyre::Result<u64> {
    // get the chain id
    let cast = Cast::new(&provider);
    let chain_id = cast.chain_id().await?;

    Ok(chain_id)
}

pub fn format_number_with_decimals(value: U256, decimals: u8) -> eyre::Result<String> {
    let num = format_units(value, decimals)?;
    let num: f64 = num.parse()?;
    if num < 10.0 {
        Ok(format!("{:.3}", num))
    } else {
        Ok(format!("{:.2}", num))
    }
}
