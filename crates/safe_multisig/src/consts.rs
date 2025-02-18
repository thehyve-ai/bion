use urls::{mainnet, sepolia};

pub mod urls {
    pub mod mainnet {
        pub const CHAIN_ID: u64 = 1;

        pub const TRANSACTION_SERVICE_URL: &str =
            "https://safe-transaction-mainnet.safe.global/api";
    }

    pub mod sepolia {
        pub const CHAIN_ID: u64 = 11155111;

        pub const TRANSACTION_SERVICE_URL: &str =
            "https://safe-transaction-sepolia.safe.global/api";
    }
}

pub fn get_transaction_service_url(chain_id: u64) -> eyre::Result<String> {
    match chain_id {
        mainnet::CHAIN_ID => Ok(mainnet::TRANSACTION_SERVICE_URL.to_string()),
        sepolia::CHAIN_ID => Ok(sepolia::TRANSACTION_SERVICE_URL.to_string()),
        _ => Err(eyre::eyre!("Chain ID not supported")),
    }
}
