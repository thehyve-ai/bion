use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use consts::get_transaction_service_url;
use transaction_data::{ProposeTransactionArgs, ProposeTransactionBody};

pub mod calls;
pub mod transaction_data;

mod consts;
mod contracts;

pub struct SafeClient {
    chain_id: u64,
    tx_service_url: String,
}

impl SafeClient {
    pub fn new(chain_id: u64) -> eyre::Result<Self> {
        let tx_service_url = get_transaction_service_url(chain_id)?;

        Ok(Self {
            chain_id,
            tx_service_url,
        })
    }

    pub async fn propose_transaction(&self, args: ProposeTransactionArgs) -> eyre::Result<()> {
        Ok(())
    }
}
