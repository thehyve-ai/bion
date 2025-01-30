use alloy_primitives::{utils::format_units, U256};
use cast::Cast;
use foundry_common::provider::RetryProvider;
use num_format::{Locale, ToFormattedString};

pub async fn get_chain_id(provider: &RetryProvider) -> eyre::Result<u64> {
    // get the chain id
    let cast = Cast::new(&provider);
    let chain_id = cast.chain_id().await?;

    Ok(chain_id)
}

pub fn parse_currency(value: U256, decimals: u8) -> eyre::Result<f64> {
    // The “value → decimal string” conversion
    let num_str = format_units(value, decimals)?;
    let float_val: f64 = num_str.parse()?;
    Ok(float_val)
}

pub fn format_number_with_decimals(value: U256, decimals: u8) -> eyre::Result<String> {
    let float_val = parse_currency(value, decimals)?;
    // Decide how many decimals to keep
    let decimal_places = if float_val < 10.0 { 3 } else { 2 };

    // Round to the chosen decimal places
    let factor = 10f64.powi(decimal_places as i32);
    let rounded = (float_val * factor).round() / factor;

    // Separate into integral and fractional parts
    let integral_part = rounded.trunc() as i64;
    let fractional_part = (rounded - integral_part as f64).abs(); // always positive portion after decimal

    // Convert the integral part with thousands separators
    let integral_str = integral_part.to_formatted_string(&Locale::en);

    // Build final string, attaching decimals properly
    if decimal_places > 0 {
        let fraction_number = (fractional_part * factor).round() as u64;
        let fractional_str = format!(
            "{:0width$}",
            fraction_number,
            width = decimal_places as usize
        );
        Ok(format!("{}.{}", integral_str, fractional_str))
    } else {
        Ok(integral_str)
    }
}
