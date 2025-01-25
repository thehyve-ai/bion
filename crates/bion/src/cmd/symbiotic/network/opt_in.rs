use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{
    cast::cmd::send::SendTxArgs, common::consts::TESTNET_ADDRESSES,
    symbiotic::calls::get_operator_network_opt_in_status, utils::validate_address_with_signer,
};

const HYVE_NETWORK_ENTITY: &str = "hyve_network";
const OPT_IN_ENTITY: &str = "network_opt_in_service";

#[derive(Debug, Parser)]
#[clap(about = "Opt in the HyveDA network.")]
pub struct OptInCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the signer."
    )]
    address: Address,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

// Weird error with async trait, so we just use a normal function
impl OptInCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        validate_address_with_signer(address, &eth).await?;

        let hyve_network = Address::from_str(TESTNET_ADDRESSES[HYVE_NETWORK_ENTITY])?;
        let opt_in_service = Address::from_str(TESTNET_ADDRESSES[OPT_IN_ENTITY])?;

        // Currently the config and provider are created twice when running the Cast command.
        // This is not ideal and should be refactored.
        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let is_opted_in =
            get_operator_network_opt_in_status(address, hyve_network, opt_in_service, &provider)
                .await?;

        if is_opted_in {
            return Err(eyre::eyre!("Address is already opted in."));
        }

        let to = foundry_common::ens::NameOrAddress::Address(opt_in_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optIn(address where)".to_string()),
            args: vec![hyve_network.to_string()],
            cast_async: true,
            confirmations,
            command: None,
            unlocked: true,
            timeout,
            tx,
            eth,
            path: None,
        };
        arg.run().await
    }
}
