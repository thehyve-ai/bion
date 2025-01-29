use addresses::{holesky, mainnet, sepolia};
use alloy_chains::Chain;
use alloy_primitives::Address;

use std::str::FromStr;

// TODO: move this to a chain spec
pub mod addresses {
    pub mod mainnet {
        pub const CHAIN_ID: u64 = 1;

        /// Pure vaults' creator (also allows their migrations)
        pub const VAULT_FACTORY: &str = "0xAEb6bdd95c502390db8f52c8909F703E9Af6a346";

        /// Pure delegators' creator
        pub const DELEGATOR_FACTORY: &str = "0x985Ed57AF9D475f1d83c1c1c8826A0E5A34E8C7B";

        /// Pure slashers' creator
        pub const SLASHER_FACTORY: &str = "0x685c2eD7D59814d2a597409058Ee7a92F21e48Fd";

        /// Networks' managing addresses (e.g., DAO contract, multisig, etc.) registrator
        pub const NETWORK_REGISTRY: &str = "0xC773b1011461e7314CF05f97d95aa8e92C1Fd8aA";

        /// Networks' metadata setter
        // pub const NETWORK_METADATA_SERVICE: &str = "0x0F7E58Cc4eA615E8B8BEB080dF8B8FDB63C21496";

        /// Networks' middleware addresses setter
        pub const NETWORK_MIDDLEWARE_SERVICE: &str = "0xD7dC9B366c027743D90761F71858BCa83C6899Ad";

        /// Operators' managing addresses (e.g., DAO contract, multisig, etc.) registrator
        pub const OPERATOR_REGISTRY: &str = "0xAd817a6Bc954F678451A71363f04150FDD81Af9F";

        /// Operators' metadata setter
        // pub const OPERATOR_METADATA_SERVICE: &str = "0x0999048aB8eeAfa053bF8581D4Aa451ab45755c9";

        /// A contract for operators' opt-ins to vaults
        pub const VAULT_OPT_IN_SERVICE: &str = "0xb361894bC06cbBA7Ea8098BF0e32EB1906A5F891";

        /// A contract for operators' opt-ins to networks
        pub const NETWORK_OPT_IN_SERVICE: &str = "0x7133415b33B438843D581013f98A08704316633c";

        /// Ready-to-work vaults' creator
        pub const VAULT_CONFIGURATOR: &str = "0x29300b1d3150B4E2b12fE80BE72f365E200441EC";
    }

    pub mod holesky {
        pub const CHAIN_ID: u64 = 17000;

        /// Pure vaults' creator (also allows their migrations)
        pub const VAULT_FACTORY: &str = "0x407A039D94948484D356eFB765b3c74382A050B4";

        /// Pure delegators' creator
        pub const DELEGATOR_FACTORY: &str = "0x890CA3f95E0f40a79885B7400926544B2214B03f";

        /// Pure slashers' creator
        pub const SLASHER_FACTORY: &str = "0xbf34bf75bb779c383267736c53a4ae86ac7bB299";

        /// Networks' managing addresses (e.g., DAO contract, multisig, etc.) registrator
        pub const NETWORK_REGISTRY: &str = "0x7d03b7343BF8d5cEC7C0C27ecE084a20113D15C9";

        /// Networks' metadata setter
        pub const NETWORK_METADATA_SERVICE: &str = "0x0F7E58Cc4eA615E8B8BEB080dF8B8FDB63C21496";

        /// Networks' middleware addresses setter
        pub const NETWORK_MIDDLEWARE_SERVICE: &str = "0x62a1ddfD86b4c1636759d9286D3A0EC722D086e3";

        /// Operators' managing addresses (e.g., DAO contract, multisig, etc.) registrator
        pub const OPERATOR_REGISTRY: &str = "0x6F75a4ffF97326A00e52662d82EA4FdE86a2C548";

        /// Operators' metadata setter
        pub const OPERATOR_METADATA_SERVICE: &str = "0x0999048aB8eeAfa053bF8581D4Aa451ab45755c9";

        /// A contract for operators' opt-ins to vaults
        pub const VAULT_OPT_IN_SERVICE: &str = "0x95CC0a052ae33941877c9619835A233D21D57351";

        /// A contract for operators' opt-ins to networks
        pub const NETWORK_OPT_IN_SERVICE: &str = "0x58973d16FFA900D11fC22e5e2B6840d9f7e13401";

        /// Ready-to-work vaults' creator
        pub const VAULT_CONFIGURATOR: &str = "0xD2191FE92987171691d552C219b8caEf186eb9cA";
    }

    pub mod sepolia {
        pub const CHAIN_ID: u64 = 11155111;

        /// Pure vaults' creator (also allows their migrations)
        pub const VAULT_FACTORY: &str = "0x407A039D94948484D356eFB765b3c74382A050B4";

        /// Pure delegators' creator
        pub const DELEGATOR_FACTORY: &str = "0x890CA3f95E0f40a79885B7400926544B2214B03f";

        /// Pure slashers' creator
        pub const SLASHER_FACTORY: &str = "0xbf34bf75bb779c383267736c53a4ae86ac7bB299";

        /// Networks' managing addresses (e.g., DAO contract, multisig, etc.) registrator
        pub const NETWORK_REGISTRY: &str = "0x7d03b7343BF8d5cEC7C0C27ecE084a20113D15C9";

        /// Networks' metadata setter
        pub const NETWORK_METADATA_SERVICE: &str = "0x0F7E58Cc4eA615E8B8BEB080dF8B8FDB63C21496";

        /// Networks' middleware addresses setter
        pub const NETWORK_MIDDLEWARE_SERVICE: &str = "0x62a1ddfD86b4c1636759d9286D3A0EC722D086e3";

        /// Operators' managing addresses (e.g., DAO contract, multisig, etc.) registrator
        pub const OPERATOR_REGISTRY: &str = "0x6F75a4ffF97326A00e52662d82EA4FdE86a2C548";

        /// Operators' metadata setter
        pub const OPERATOR_METADATA_SERVICE: &str = "0x0999048aB8eeAfa053bF8581D4Aa451ab45755c9";

        /// A contract for operators' opt-ins to vaults
        pub const VAULT_OPT_IN_SERVICE: &str = "0x95CC0a052ae33941877c9619835A233D21D57351";

        /// A contract for operators' opt-ins to networks
        pub const NETWORK_OPT_IN_SERVICE: &str = "0x58973d16FFA900D11fC22e5e2B6840d9f7e13401";

        /// Ready-to-work vaults' creator
        pub const VAULT_CONFIGURATOR: &str = "0xD2191FE92987171691d552C219b8caEf186eb9cA";
    }
}

pub fn get_vault_factory(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::VAULT_FACTORY)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::VAULT_FACTORY)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::VAULT_FACTORY)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_delegator_factory(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::DELEGATOR_FACTORY)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::DELEGATOR_FACTORY)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::DELEGATOR_FACTORY)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_slasher_factory(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::SLASHER_FACTORY)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::SLASHER_FACTORY)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::SLASHER_FACTORY)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_network_registry(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::NETWORK_REGISTRY)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::NETWORK_REGISTRY)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::NETWORK_REGISTRY)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_network_middleware_service(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::NETWORK_MIDDLEWARE_SERVICE)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::NETWORK_MIDDLEWARE_SERVICE)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::NETWORK_MIDDLEWARE_SERVICE)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_operator_registry(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::OPERATOR_REGISTRY)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::OPERATOR_REGISTRY)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::OPERATOR_REGISTRY)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_vault_opt_in_service(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::VAULT_OPT_IN_SERVICE)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::VAULT_OPT_IN_SERVICE)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::VAULT_OPT_IN_SERVICE)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_network_opt_in_service(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::NETWORK_OPT_IN_SERVICE)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::NETWORK_OPT_IN_SERVICE)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::NETWORK_OPT_IN_SERVICE)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}

pub fn get_vault_configurator(chain_id: u64) -> eyre::Result<Address> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(Address::from_str(mainnet::VAULT_CONFIGURATOR)?),
        holesky::CHAIN_ID => Ok(Address::from_str(holesky::VAULT_CONFIGURATOR)?),
        sepolia::CHAIN_ID => Ok(Address::from_str(sepolia::VAULT_CONFIGURATOR)?),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}
