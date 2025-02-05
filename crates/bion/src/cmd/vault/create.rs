use alloy_primitives::{aliases::U48, Address, Bytes, U256};
use alloy_sol_types::SolValue;
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
        consts::get_vault_configurator,
        contracts::{vault_configurator::IVaultConfigurator, IVault},
    },
    utils::{print_error_message, print_success_message, validate_cli_args},
};

use super::utils::{get_vault_admin_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
#[clap(about = "Create a new vault with a delegator and a slasher.")]
pub struct CreateCommand {
    // Vault params
    #[arg(
        value_name = "VERSION",
        help = "Version of the vault. 1 - Common; 2 - Tokenized."
    )]
    version: u64,

    #[arg(value_name = "COLLATERAL", help = "Address of the collateral.")]
    collateral: Address,

    #[arg(value_name = "BURNER", help = "Address of the deployed burner router.")]
    burner: Address,

    #[arg(
        value_name = "EPOCH_DURATION",
        help = "Duration of the Vault epoch in seconds."
    )]
    epoch_duration: U48,

    #[arg(
        value_name = "DEPOSIT_WHITELIST",
        help = "Enable deposit whitelisting."
    )]
    deposit_whitelist: bool,

    #[arg(value_name = "IS_DEPOSIT_LIMIT", help = "Enable deposit limit.")]
    is_deposit_limit: bool,

    #[arg(value_name = "DEPOSIT_LIMIT", help = "The deposit limit.")]
    deposit_limit: U256,

    // Delegator params
    #[arg(
        value_name = "DELEGATOR_INDEX",
        help = "Type of the Delegator. 0 - NetworkRestakeDelegator; 1 - FullRestakeDelegator; 2 - OperatorSpecificDelegator; 3 - OperatorNetworkSpecificDelegator."
    )]
    delegator_index: u64,

    #[arg(value_name = "DELEGATOR_HOOK", help = "Address of the Delegator hook.")]
    delegator_hook: Address,

    // Slasher params
    #[arg(value_name = "WITH_SLASHER", help = "Enables the Slasher module.")]
    with_slasher: bool,

    #[arg(
        value_name = "SLASHER_INDEX",
        help = "Type of the Slasher. 0 - Slasher; 1 - VetoSlasher."
    )]
    slasher_index: u64,

    #[arg(value_name = "VETO_DURATION", help = "Veto duration in seconds.")]
    veto_duration: u64,

    #[arg(
        value_name = "RESOLVER_SET_EPOCHS_DELAY",
        help = "The number of Vault epochs needed for the resolver to be changed."
    )]
    resolver_set_epochs_delay: u64,

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

impl CreateCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            version,
            collateral,
            burner,
            epoch_duration,
            deposit_whitelist,
            is_deposit_limit,
            deposit_limit,
            delegator_index,
            delegator_hook,
            with_slasher,
            slasher_index,
            veto_duration,
            resolver_set_epochs_delay,
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
        let vault_admin_config = get_vault_admin_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        let vault_configurator = get_vault_configurator(chain_id)?;

        let network_limit_set_role_holders = vec![vault_admin_config.address];
        let operator_network_shares_set_role_holders = vec![vault_admin_config.address];

        let vault_params = IVault::InitParams {
            collateral,
            burner,
            epochDuration: epoch_duration,
            depositWhitelist: deposit_whitelist,
            isDepositLimit: is_deposit_limit,
            depositLimit: deposit_limit,
            defaultAdminRoleHolder: vault_admin_config.address,
            depositWhitelistSetRoleHolder: vault_admin_config.address,
            depositorWhitelistRoleHolder: vault_admin_config.address,
            isDepositLimitSetRoleHolder: vault_admin_config.address,
            depositLimitSetRoleHolder: vault_admin_config.address,
        };
        let delegator_params: Vec<u8> = vec![];
        let slasher_params: Vec<u8> = vec![];

        let configurator_init_params = IVaultConfigurator::InitParams {
            version,
            owner: vault_admin_config.address,
            vaultParams: vault_params.abi_encode().into(),
            delegatorIndex: delegator_index,
            delegatorParams: delegator_params.into(),
            withSlasher: with_slasher,
            slasherIndex: slasher_index,
            slasherParams: slasher_params.into(),
        };

        let to = NameOrAddress::Address(vault_configurator);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("create()".to_string()),
            args: vec![],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        if let Ok(..) = arg.run().await {
            print_success_message("✅ Successfully created vault.");
        } else {
            print_error_message("❌ Failed to create vault, please try again.");
        }
        Ok(())
    }
}
