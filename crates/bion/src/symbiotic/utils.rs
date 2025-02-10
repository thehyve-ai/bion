use alloy_primitives::{aliases::U96, Address, FixedBytes};

use std::str::FromStr;

pub fn get_subnetwork(network: Address, subnetwork: U96) -> eyre::Result<FixedBytes<32>> {
    let subnetwork = format!("{}{}", network, subnetwork);
    let subnetwork = FixedBytes::from_str(&subnetwork)?;
    Ok(subnetwork)
}
