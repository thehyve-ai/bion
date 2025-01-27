// TODO: move this to a chain spec
pub mod addresses {
    pub mod mainnet {}

    pub mod sepolia {
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
