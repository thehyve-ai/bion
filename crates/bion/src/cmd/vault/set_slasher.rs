use alloy_primitives::Address;
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
    symbiotic::{
        consts::{get_slasher_factory, get_vault_factory},
        vault_utils::{validate_slasher_symbiotic_status, validate_vault_symbiotic_status},
    },
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Set the slasher for your vault.")]
pub struct SetSlasherCommand {
    #[arg(value_name = "VAULT", help = "Address of the vault.")]
    vault: Address,

    #[arg(value_name = "SLASHER", help = "Address of the slasher.")]
    slasher: Address,

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

impl SetSlasherCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            slasher,
            alias,
            dirs,
            mut eth,
            tx,
            unlocked,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let vault_admin_config = get_alias_config(chain_id, alias, &dirs)?;
        let slasher_factory = get_slasher_factory(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        validate_vault_symbiotic_status(vault, vault_factory, &provider).await?;
        validate_slasher_symbiotic_status(slasher, slasher_factory, &provider).await?;

        let to = NameOrAddress::Address(vault);

        let args = SendTxArgs {
            to: Some(to),
            sig: Some("setSlasher(address slasher)".to_string()),
            args: vec![slasher.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth: eth.clone(),
            path: None,
        };

        match vault_admin_config.signing_method {
            Some(SigningMethod::MultiSig) => {
                let safe = SafeClient::new(chain_id)?;
                let signer = eth.wallet.signer().await?;
                let mut executable_args = args.clone();
                if let Some(ExecutableSafeTransaction {
                    safe_address,
                    input_data,
                }) = safe
                    .send_tx(
                        vault_admin_config.address,
                        signer,
                        args.try_into()?,
                        &provider,
                    )
                    .await?
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
