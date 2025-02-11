use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::get_chain_id,
    common::DirsCliArgs,
    symbiotic::{
        calls::is_opted_in_network,
        consts::{get_network_opt_in_service, get_network_registry},
        network_utils::validate_network_status,
    },
    utils::{print_loading_until_async, validate_cli_args},
};

use super::utils::{get_operator_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
pub struct OptInNetworkCommand {
    #[arg(value_name = "NETWORK", help = "The address of the network.")]
    network: Address,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,

    #[clap(flatten)]
    tx: TransactionOpts,

    /// Send via `eth_sendTransaction using the `--from` argument or $ETH_FROM as sender
    #[arg(long, requires = "from")]
    pub unlocked: bool,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

impl OptInNetworkCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            network,
            alias,
            dirs,
            mut eth,
            tx,
            confirmations,
            timeout,
            unlocked,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;
        let opt_in_service = get_network_opt_in_service(chain_id)?;
        let operator_config = get_operator_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&operator_config, &mut eth)?;

        validate_network_status(network, network_registry, &provider).await?;

        let is_opted_in = print_loading_until_async(
            "Checking opted in status",
            is_opted_in_network(operator_config.address, network, opt_in_service, &provider),
        )
        .await?;

        if is_opted_in {
            return Err(eyre::eyre!("Operator is already opted in."));
        }

        let to = NameOrAddress::Address(opt_in_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optIn(address where)".to_string()),
            args: vec![network.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        let _ = arg.run().await?;
        Ok(())
    }
}
