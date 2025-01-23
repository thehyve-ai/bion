use phf::phf_map;

pub static SEPOLIA_CHAIN_ID: u64 = 11155111;

pub static TESTNET_RPC_ENDPOINT: &str =
    "https://sepolia.infura.io/v3/2adcbf1a2dd9449382175510fbf4f188";

pub static TESTNET_ADDRESSES: phf::Map<&'static str, &'static str> = phf_map! {
    "hyve_network" => "0x4709d01007788ecfef90a015144f4e278d498736",
    "hyve_middleware_service" => "0x1bCc35C944Dc2D3e4990942243ed89c403b1888a",
    "network_opt_in_service" => "0x58973d16FFA900D11fC22e5e2B6840d9f7e13401",
    "op_registry" => "0x6F75a4ffF97326A00e52662d82EA4FdE86a2C548",
    "vault_opt_in_service" => "0x95CC0a052ae33941877c9619835A233D21D57351"
};

pub static TESTNET_VAULTS: phf::Map<&'static str, &'static str> = phf_map! {
    "wstETH" => "0x1BAe55e4774372F6181DaAaB4Ca197A8D9CC06Dd",
};

pub static DELEGATOR_TYPES_ENTITIES: phf::Map<&'static str, &'static str> = phf_map! {
    "network" => "network_restake_delegator",
    "full" => "full_restake_delegator",
};

pub static DELEGATOR_TYPES_NAMES: phf::Map<&'static str, &'static str> = phf_map! {
    "network" => "NetworkRestake",
    "full" => "FullRestake",
};

pub static SLASHER_TYPES_NAMES: phf::Map<&'static str, &'static str> = phf_map! {
    "non" => "NonSlashable",
    "instant" => "InstantSlasher",
    "veto" => "VetoSlasher",
};
