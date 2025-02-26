use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;
use safe_multisig::SafeClient;

use crate::{
    cast::{cmd::send::SendTxArgs, utils::build_tx},
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::{DirsCliArgs, SigningMethod},
    symbiotic::{
        consts::{get_network_middleware_service, get_network_registry},
        network_utils::validate_network_status,
    },
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Set the middleware for your network.")]
pub struct SetMiddlewareCommand {
    #[arg(
        value_name = "MIDDLEWARE_ADDRESS",
        help = "The address of the network middleware."
    )]
    middleware_address: Address,

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

impl SetMiddlewareCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            middleware_address,
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
        let network_config = get_alias_config(chain_id, alias, &dirs)?;
        let network = network_config.address;
        let network_registry = get_network_registry(chain_id)?;
        let network_middleware_service = get_network_middleware_service(chain_id)?;
        set_foundry_signing_method(&network_config, &mut eth)?;

        validate_network_status(network, network_registry, &provider).await?;

        let to = NameOrAddress::Address(network_middleware_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setMiddleware(address middleware_".to_string()),
            args: vec![middleware_address.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth: eth.clone(),
            path: None,
        };

        match network_config.signing_method {
            Some(SigningMethod::MultiSig) => {
                let safe = SafeClient::new(chain_id)?;
                let signer = eth.wallet.signer().await?;
                let tx = build_tx(arg, &config, &provider).await?;
                safe.send_tx(network_config.address, signer, tx, &provider)
                    .await?;
            }
            _ => {
                let _ = arg.run().await?;
            }
        };
        Ok(())
    }
}
