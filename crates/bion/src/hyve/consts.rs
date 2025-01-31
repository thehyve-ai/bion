use addresses::{holesky, mainnet, sepolia};
use alloy_primitives::Address;

use std::str::FromStr;

pub mod addresses {
    pub mod mainnet {
        pub const CHAIN_ID: u64 = 1;

        pub const HYVE_NETWORK: &str = "0xE3a148b25Cca54ECCBD3A4aB01e235D154f03eFa";

        pub const HYVE_MIDDLEWARE_SERVICE: &str = "0xBf3e64f0f83d5ce7f9BfFA0eE7Ec524e9999D572";
    }

    pub mod holesky {
        pub const CHAIN_ID: u64 = 17000;

        pub const HYVE_NETWORK: &str = "0x";

        pub const HYVE_MIDDLEWARE_SERVICE: &str = "0x";
    }

    pub mod sepolia {
        pub const CHAIN_ID: u64 = 11155111;

        pub const HYVE_NETWORK: &str = "0x4709d01007788ecfef90a015144f4e278d498736";

        pub const HYVE_MIDDLEWARE_SERVICE: &str = "0x1bCc35C944Dc2D3e4990942243ed89c403b1888a";
    }
}

pub fn get_hyve_network(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::HYVE_NETWORK)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::HYVE_NETWORK)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::HYVE_NETWORK)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_hyve_middleware_service(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::HYVE_MIDDLEWARE_SERVICE)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::HYVE_MIDDLEWARE_SERVICE)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::HYVE_MIDDLEWARE_SERVICE)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}
