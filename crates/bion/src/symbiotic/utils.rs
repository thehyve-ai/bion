use alloy_primitives::{aliases::U96, Address, FixedBytes, U256};

pub fn get_vault_link(vault: Address, text: String) -> String {
    format!(
        "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
        vault, text
    )
}

pub fn get_network_link(network: Address, text: String) -> String {
    format!(
        "\x1B]8;;https://app.symbiotic.fi/network/{}\x1B\\{}\x1B]8;;\x1B\\",
        network, text
    )
}

pub fn get_subnetwork(network: Address, subnetwork: U96) -> eyre::Result<FixedBytes<32>> {
    let address_u256 = U256::from_be_bytes(network.into_word().0);
    // shift the address (160 bits) left by 96 bits
    let shifted = address_u256 << 96;
    // bitwise OR with the U96
    let combined: U256 = shifted | U256::from(subnetwork);

    // convert to big-endian bytes, then wrap in FixedBytes<32>
    let combined_bytes = combined.to_be_bytes();
    Ok(FixedBytes::<32>::from(combined_bytes))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::B256;

    use super::*;

    #[test]
    fn test_get_subnetwork() {
        let expected_subnetwork_0 =
            B256::from_str("0x1234567890123456789012345678901234567890000000000000000000000000")
                .unwrap();
        let expected_subnetwork_1 =
            B256::from_str("0x1234567890123456789012345678901234567890000000000000000000000001")
                .unwrap();
        let expected_subnetwork_2 =
            B256::from_str("0x1234567890123456789012345678901234567890000000000000000000000002")
                .unwrap();
        let expected_subnetwork_34 =
            B256::from_str("0x1234567890123456789012345678901234567890000000000000000000000022")
                .unwrap();
        let expected_subnetwork_200 =
            B256::from_str("0x12345678901234567890123456789012345678900000000000000000000000c8")
                .unwrap();

        let network = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
        let subnetwork_0 = get_subnetwork(network, U96::from(0)).unwrap();
        let subnetwork_1 = get_subnetwork(network, U96::from(1)).unwrap();
        let subnetwork_2 = get_subnetwork(network, U96::from(2)).unwrap();
        let subnetwork_34 = get_subnetwork(network, U96::from(34)).unwrap();
        let subnetwork_200 = get_subnetwork(network, U96::from(200)).unwrap();

        assert_eq!(subnetwork_0, expected_subnetwork_0);
        assert_eq!(subnetwork_1, expected_subnetwork_1);
        assert_eq!(subnetwork_2, expected_subnetwork_2);
        assert_eq!(subnetwork_34, expected_subnetwork_34);
        assert_eq!(subnetwork_200, expected_subnetwork_200);
    }
}
