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

#[cfg(test)]
mod test {
    use alloy::node_bindings::Anvil;
    use alloy_network::EthereumWallet;
    use alloy_provider::{ProviderBuilder, WsConnect};
    use alloy_signer_local::PrivateKeySigner;
    use alloy_sol_types::sol;

    use crate::symbiotic::contracts::IOperatorRegistry;

    sol!(
        #[sol(rpc, bytecode = "60808060405234601557610241908161001a8239f35b5f80fdfe60806040526004361015610011575f80fd5b5f3560e01c806314887c581461013d5780632acde098146100d15780635cd8b15e146100b55763b42ba2a214610045575f80fd5b346100b15760203660031901126100b1576004355f5481101561009d575f80527f290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e56301546040516001600160a01b039091168152602090f35b634e487b7160e01b5f52603260045260245ffd5b5f80fd5b346100b1575f3660031901126100b15760205f54604051908152f35b346100b1575f3660031901126100b1576100f6335f52600160205260405f2054151590565b61012b5761010333610184565b50337fb919910dcefbf753bfd926ab3b1d3f85d877190c3d01ba1bd585047b99b99f0b5f80a2005b6040516342ee68b560e01b8152600490fd5b346100b15760203660031901126100b1576004356001600160a01b038116908190036100b15761017a6020915f52600160205260405f2054151590565b6040519015158152f35b805f52600160205260405f2054155f14610206575f54680100000000000000008110156101f25760018101805f5581101561009d5781907f290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e56301555f54905f52600160205260405f2055600190565b634e487b7160e01b5f52604160045260245ffd5b505f9056fea264697066735822122075b5716387895e629e74b8e3c027010c3b81a41d26ee7b556bbe4c038045108064736f6c63430008190033")]
        contract OperatorRegistry {

        }
    );

    #[tokio::test]
    async fn test_register_operator() {
        let anvil = Anvil::new().block_time(1).try_spawn().unwrap();

        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer.clone());

        let rpc_url = anvil.ws_endpoint_url();
        let ws_url = WsConnect::new(rpc_url.clone());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_ws(ws_url)
            .await
            .unwrap();

        let operator_registry = OperatorRegistry::deploy(provider.clone()).await.unwrap();
        let operator_registry =
            IOperatorRegistry::new(operator_registry.address().clone(), provider.clone());

        let pending_tx =
            operator_registry.registerOperator().send().await.unwrap().register().await.unwrap();
        let _ = pending_tx.await;

        let IOperatorRegistry::isEntityReturn { _0: is_registered } =
            operator_registry.isEntity(signer.address()).call().await.unwrap();
        assert!(is_registered);
    }
}
