use alloy_primitives::Address;
use foundry_cli::opts::EthereumOpts;
use prettytable::Table;

use std::fs::create_dir_all;

use crate::{
    common::{DirsCliArgs, SigningMethod},
    utils::{load_from_json_file, print_error_message, write_to_json_file},
};

use super::{
    config::{ImportedOperators, OperatorConfig},
    consts::{OPERATOR_CONFIG_FILE, OPERATOR_DEFINITIONS_FILE, OPERATOR_DIRECTORY},
};

pub fn get_or_create_operator_definitions(
    chain_id: u64,
    dirs: &DirsCliArgs,
) -> eyre::Result<ImportedOperators> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let operators_dir = data_dir.join(OPERATOR_DIRECTORY);
    let operator_definitions_path = operators_dir.join(OPERATOR_DEFINITIONS_FILE);
    return match load_from_json_file(&operator_definitions_path) {
        Ok(operators_map) => Ok(operators_map),
        Err(..) => {
            create_dir_all(&operators_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create operator directory: {:?}: {:?}",
                    operator_definitions_path, e
                ))
            })?;

            let operators_map = ImportedOperators::new();
            write_to_json_file(&operator_definitions_path, &operators_map, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(operators_map)
        }
    };
}

pub fn get_or_create_operator_config(
    chain_id: u64,
    address: Address,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<OperatorConfig> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let operators_dir = data_dir.join(OPERATOR_DIRECTORY);
    let operator_config_dir = operators_dir.join(address.to_string());
    let operator_config_path = operator_config_dir.join(OPERATOR_CONFIG_FILE);
    return match load_from_json_file(&operator_config_path) {
        Ok(operator_config) => Ok(operator_config),
        Err(..) => {
            create_dir_all(&operator_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create operator config directory: {:?}: {:?}",
                    operator_config_dir, e
                ))
            })?;

            let operator_config = OperatorConfig::new(address, chain_id, alias);
            write_to_json_file(&operator_config_path, &operator_config, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(operator_config)
        }
    };
}

pub fn get_operator_config(
    chain_id: u64,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<OperatorConfig> {
    let operator_definitions = get_or_create_operator_definitions(chain_id, dirs)?;
    if let Some((_, address)) = operator_definitions.get_key_value(&alias) {
        let operator_config = get_or_create_operator_config(chain_id, *address, alias, dirs)?;
        Ok(operator_config)
    } else {
        print_error_message("Operator with the provided alias is not imported.");
        Err(eyre::eyre!(""))
    }
}

pub fn set_foundry_signing_method(
    operator_config: &OperatorConfig,
    eth: &mut EthereumOpts,
) -> eyre::Result<()> {
    if let Some(signing_method) = operator_config.signing_method.clone() {
        match signing_method {
            SigningMethod::Keystore => {
                let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;
                eth.wallet.keystore_path = Some(
                    operator_config
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
