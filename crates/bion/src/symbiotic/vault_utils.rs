use alloy_primitives::{aliases::U48, hex::ToHexExt, Address, Bytes, U256};
use alloy_sol_types::SolValue;

use crate::{cmd::vault::config::VaultAdminConfig, utils::print_error_message};

use super::contracts::{
    delegator::{
        base_delegator::IBaseDelegator, full_restake_delegator::IFullRestakeDelegator,
        network_restake_delegator::INetworkRestakeDelegator,
        operator_network_specific_delegator::IOperatorNetworkSpecificDelegator,
        operator_specific_delegator::IOperatorSpecificDelegator,
    },
    slasher::{base_slasher::IBaseSlasher, slasher::ISlasher, veto_slasher::IVetoSlasher},
    vault_configurator::IVaultConfigurator,
    IVault,
};

pub fn get_encoded_vault_configurator_params(
    version: u64,
    collateral: Address,
    burner: Option<Address>,
    epoch_duration: U48,
    deposit_whitelist: bool,
    is_deposit_limit: bool,
    deposit_limit: U256,
    delegator_index: u64,
    delegator_hook: Option<Address>,
    with_slasher: bool,
    slasher_index: u64,
    veto_duration: U48,
    resolver_set_epochs_delay: U256,
    vault_admin_config: &VaultAdminConfig,
) -> eyre::Result<String> {
    let network_limit_set_role_holders = vec![vault_admin_config.address];
    let operator_network_shares_set_role_holders = vec![vault_admin_config.address];

    let burner = burner.unwrap_or(Address::ZERO);
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
    let delegator_params: Vec<u8> = match delegator_index {
        // NetworkRestakeDelegator (type 0)
        0 => INetworkRestakeDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            networkLimitSetRoleHolders: network_limit_set_role_holders,
            operatorNetworkSharesSetRoleHolders: operator_network_shares_set_role_holders,
        }
        .abi_encode(),

        // FullRestakeDelegator (type 1)
        1 => IFullRestakeDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            networkLimitSetRoleHolders: network_limit_set_role_holders,
            operatorNetworkLimitSetRoleHolders: operator_network_shares_set_role_holders,
        }
        .abi_encode(),

        // OperatorSpecificDelegator (type 2)
        2 => IOperatorSpecificDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            networkLimitSetRoleHolders: network_limit_set_role_holders,
            operator: Address::ZERO,
        }
        .abi_encode(),

        // OperatorNetworkSpecificDelegator (type 3)
        3 => IOperatorNetworkSpecificDelegator::InitParams {
            baseParams: IBaseDelegator::BaseParams {
                defaultAdminRoleHolder: vault_admin_config.address,
                hook: delegator_hook.unwrap_or(Address::ZERO),
                hookSetRoleHolder: vault_admin_config.address,
            },
            network: Address::ZERO,
            operator: Address::ZERO,
        }
        .abi_encode(),
        _ => {
            print_error_message("Invalid delegator index.");
            return Err(eyre::eyre!(""));
        }
    };

    let mut slasher_params = vec![];
    if with_slasher {
        slasher_params = match slasher_index {
            // Slasher (type 0)
            0 => ISlasher::InitParams {
                baseParams: IBaseSlasher::BaseParams {
                    isBurnerHook: !burner.is_zero(),
                },
            }
            .abi_encode(),

            // VetoSlasher (type 1)
            1 => IVetoSlasher::InitParams {
                baseParams: IBaseSlasher::BaseParams {
                    isBurnerHook: !burner.is_zero(),
                },
                vetoDuration: veto_duration,
                resolverSetEpochsDelay: resolver_set_epochs_delay,
            }
            .abi_encode(),
            _ => {
                print_error_message("Invalid slasher index.");
                return Err(eyre::eyre!(""));
            }
        };
    }

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

    let configurator_params_bytes: Bytes = configurator_init_params.abi_encode_params().into();
    let encoded = format!("0x{}", configurator_params_bytes.encode_hex());
    Ok(encoded)
}
