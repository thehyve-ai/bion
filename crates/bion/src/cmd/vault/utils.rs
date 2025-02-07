use alloy_primitives::Address;
use foundry_cli::opts::EthereumOpts;

use std::fs::create_dir_all;

use crate::{
    common::{DirsCliArgs, SigningMethod},
    utils::{load_from_json_file, print_error_message, write_to_json_file},
};

use super::{
    config::{ImportedVaultAdmins, VaultAdminConfig},
    consts::{VAULT_CONFIG_FILE, VAULT_DEFINITIONS_FILE, VAULT_DIRECTORY},
};

pub fn get_or_create_vault_admin_definitions(
    chain_id: u64,
    dirs: &DirsCliArgs,
) -> eyre::Result<ImportedVaultAdmins> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let vault_dir = data_dir.join(VAULT_DIRECTORY);
    let vault_definitions_path = vault_dir.join(VAULT_DEFINITIONS_FILE);
    return match load_from_json_file(&vault_definitions_path) {
        Ok(vault_map) => Ok(vault_map),
        Err(..) => {
            create_dir_all(&vault_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create vault directory: {:?}: {:?}",
                    vault_definitions_path, e
                ))
            })?;

            let vault_admin_map = ImportedVaultAdmins::new();
            write_to_json_file(&vault_definitions_path, &vault_admin_map, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(vault_admin_map)
        }
    };
}

pub fn get_or_create_vault_admin_config(
    chain_id: u64,
    address: Address,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<VaultAdminConfig> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let vault_dir = data_dir.join(VAULT_DIRECTORY);
    let vault_config_dir = vault_dir.join(address.to_string());
    let vault_config_path = vault_config_dir.join(VAULT_CONFIG_FILE);
    return match load_from_json_file(&vault_config_path) {
        Ok(vault_config) => Ok(vault_config),
        Err(..) => {
            create_dir_all(&vault_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create vault config directory: {:?}: {:?}",
                    vault_config_dir, e
                ))
            })?;

            let vault_config = VaultAdminConfig::new(address, chain_id, alias);
            write_to_json_file(&vault_config_path, &vault_config, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(vault_config)
        }
    };
}

pub fn get_vault_admin_config(
    chain_id: u64,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<VaultAdminConfig> {
    let vault_definitions = get_or_create_vault_admin_definitions(chain_id, dirs)?;
    if let Some((_, address)) = vault_definitions.get_key_value(&alias) {
        let vault_config = get_or_create_vault_admin_config(chain_id, *address, alias, dirs)?;
        Ok(vault_config)
    } else {
        print_error_message("Vault admin with the provided alias is not imported.");
        Err(eyre::eyre!(""))
    }
}

pub fn set_foundry_signing_method(
    vault_admin_config: &VaultAdminConfig,
    eth: &mut EthereumOpts,
) -> eyre::Result<()> {
    if let Some(signing_method) = vault_admin_config.signing_method.clone() {
        match signing_method {
            SigningMethod::Keystore => {
                let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;
                eth.wallet.keystore_path = Some(
                    vault_admin_config
                        .keystore_file
                        .clone()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                );
                eth.wallet.keystore_password = Some(password);
            }
            SigningMethod::Ledger => {
                eth.wallet.ledger = true;
            }
            SigningMethod::Trezor => {
                eth.wallet.trezor = true;
            }
        }
    }
    Ok(())
}
