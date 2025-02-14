use alloy_primitives::Address;
use foundry_cli::opts::EthereumOpts;

use std::fs::create_dir_all;

use crate::{
    common::{DirsCliArgs, SigningMethod},
    utils::{load_from_json_file, write_to_json_file},
};

use super::{
    alias_config::{AliasConfig, ImportedAliases},
    consts::{ALIAS_CONFIG_FILE, ALIAS_DEFINITIONS_FILE, ALIAS_DIRECTORY},
};

pub fn get_or_create_alias_definitions(
    chain_id: u64,
    dirs: &DirsCliArgs,
) -> eyre::Result<ImportedAliases> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let alias_dir = data_dir.join(ALIAS_DIRECTORY);
    let alias_definitions_path = alias_dir.join(ALIAS_DEFINITIONS_FILE);
    return match load_from_json_file(&alias_definitions_path) {
        Ok(alias_map) => Ok(alias_map),
        Err(..) => {
            create_dir_all(&alias_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create alias directory: {:?}: {:?}",
                    alias_definitions_path, e
                ))
            })?;

            let alias_map = ImportedAliases::new();
            write_to_json_file(&alias_definitions_path, &alias_map, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(alias_map)
        }
    };
}

pub fn get_or_create_alias_config(
    chain_id: u64,
    address: Address,
    alias: String,
    dirs: &DirsCliArgs,
    fail_on_error: bool,
) -> eyre::Result<AliasConfig> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let alias_dir = data_dir.join(ALIAS_DIRECTORY);
    let alias_config_dir = alias_dir.join(address.to_string());
    let alias_config_path = alias_config_dir.join(ALIAS_CONFIG_FILE);
    return match load_from_json_file(&alias_config_path) {
        Ok(alias_config) => Ok(alias_config),
        Err(err) => {
            if fail_on_error {
                eyre::bail!(err);
            }

            create_dir_all(&alias_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create alias config directory: {:?}: {:?}",
                    alias_config_dir, e
                ))
            })?;

            let alias_config = AliasConfig::new(address, chain_id, alias);
            write_to_json_file(&alias_config_path, &alias_config, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(alias_config)
        }
    };
}

pub fn get_alias_config(
    chain_id: u64,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<AliasConfig> {
    let alias_definitions = get_or_create_alias_definitions(chain_id, dirs)?;
    if let Some((_, address)) = alias_definitions.get_key_value(&alias) {
        let alias_config = get_or_create_alias_config(chain_id, *address, alias, dirs, true)?;
        Ok(alias_config)
    } else {
        eyre::bail!("Alias is not imported.");
    }
}

pub fn set_foundry_signing_method(
    alias_config: &AliasConfig,
    eth: &mut EthereumOpts,
) -> eyre::Result<()> {
    if let Some(signing_method) = alias_config.signing_method.clone() {
        match signing_method {
            SigningMethod::Keystore => {
                let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;
                eth.wallet.keystore_path = Some(
                    alias_config
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
