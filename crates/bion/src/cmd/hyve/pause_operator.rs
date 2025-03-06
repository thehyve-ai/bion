use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    hyve::{
        consts::get_hyve_middleware_service,
        operator_utils::validate_operator_hyve_middleware_status,
    },
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Pauses an Operator in the HyveDA middleware.")]
pub struct PauseOperatorCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

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

impl PauseOperatorCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            alias,
            dirs,
            tx,
            mut eth,
            unlocked,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let operator_config = get_alias_config(chain_id, alias, &dirs)?;
        let operator = operator_config.address;
        let hyve_middleware = get_hyve_middleware_service(chain_id)?;
        set_foundry_signing_method(&operator_config, &mut eth)?;

        // validate that the operator is eligible for pausing
        // get the timestamp for epoch + 1
        // if the operator is active in epoch + 1 then we can pause
        // figure out how to deal with immutable period error

        validate_operator_hyve_middleware_status(operator, hyve_middleware, &provider).await?;

        let to = foundry_common::ens::NameOrAddress::Address(hyve_middleware);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("pauseOperator()".to_string()),
            args: vec![],
            cast_async: true,
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
