use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;
use safe_multisig::{transaction_data::ExecutableSafeTransaction, SafeClient};

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::{DirsCliArgs, SigningMethod},
    symbiotic::{calls::is_operator, consts::get_operator_registry},
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Register your operator.")]
pub struct RegisterCommand {
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

impl RegisterCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { alias, dirs, mut eth, tx, confirmations, timeout, unlocked } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let operator_config = get_alias_config(chain_id, alias, &dirs)?;
        let operator = operator_config.address;
        let operator_registry = get_operator_registry(chain_id)?;
        set_foundry_signing_method(&operator_config, &mut eth)?;

        let is_registered = print_loading_until_async(
            "Checking registration status",
            is_operator(operator, operator_registry, &provider),
        )
        .await?;

        if is_registered {
            eyre::bail!("Operator is already registered");
        }

        let to = NameOrAddress::Address(operator_registry);

        let args = SendTxArgs {
            to: Some(to),
            sig: Some("registerOperator()".to_string()),
            args: vec![],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth: eth.clone(),
            path: None,
        };

        match operator_config.signing_method {
            Some(SigningMethod::MultiSig) => {
                let safe = SafeClient::new(chain_id)?;
                let signer = eth.wallet.signer().await?;
                let mut executable_args = args.clone();
                if let Some(ExecutableSafeTransaction { safe_address, input_data }) =
                    safe.send_tx(operator, signer, args.try_into()?, &provider).await?
                {
                    executable_args.to = Some(NameOrAddress::Address(safe_address));
                    executable_args.sig = Some(input_data);
                    let _ = executable_args.run().await?;
                }
            }
            _ => {
                let _ = args.run().await?;
            }
        };
        Ok(())
    }
}
