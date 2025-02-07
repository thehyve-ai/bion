use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::get_chain_id,
    hyve::consts::get_hyve_network,
    symbiotic::{calls::is_opted_in_network, consts::get_network_opt_in_service},
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Opt out of a Symbiotic network.")]
pub struct NetworkOptOutCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the operator."
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

impl NetworkOptOutCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let hyve_network = get_hyve_network(chain_id)?;
        let opt_in_service = get_network_opt_in_service(chain_id)?;

        let is_opted_in =
            is_opted_in_network(address, hyve_network, opt_in_service, &provider).await?;

        if !is_opted_in {
            return Err(eyre::eyre!(
                "Cannot opt-out of network because the operator is not opted-in."
            ));
        }

        let to = foundry_common::ens::NameOrAddress::Address(opt_in_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optOut(address where)".to_string()),
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
        arg.run().await?;
        Ok(())
    }
}
