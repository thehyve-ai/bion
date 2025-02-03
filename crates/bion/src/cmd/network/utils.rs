use alloy_primitives::Address;
use foundry_cli::opts::EthereumOpts;
use serde::Deserialize;

use std::fs::create_dir_all;

use crate::{
    common::{DirsCliArgs, SigningMethod},
    utils::{load_from_json_file, print_error_message, write_to_json_file},
};

use super::{
    config::{ImportedNetworks, NetworkConfig},
    consts::{
        NETWORK_CONFIG_FILE, NETWORK_DEFINITIONS_FILE, NETWORK_DIRECTORY, SYMBIOTIC_GITHUB_URL,
        SYMBIOTIC_NETWORK_FILE_NAME,
    },
};

#[derive(Debug, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
}

/// Fetches metadata for a Symbiotic network from the official GitHub repository
///
/// # Arguments
/// * `network_address` - The address of the network to fetch metadata for
///
/// # Returns
/// * `NetworkInfo` containing the network's metadata
///
/// # Errors
/// * If the HTTP request fails
/// * If the response cannot be parsed as JSON
/// * If the JSON cannot be deserialized into `VaultInfo`
pub async fn get_network_metadata(network_address: Address) -> eyre::Result<Option<NetworkInfo>> {
    let url = format!("{SYMBIOTIC_GITHUB_URL}/{network_address}/{SYMBIOTIC_NETWORK_FILE_NAME}",);
    let res = reqwest::get(&url).await?;
    let vault_info: Option<NetworkInfo> = serde_json::from_str(&res.text().await?).ok();
    Ok(vault_info)
}

pub fn get_or_create_network_definitions(
    chain_id: u64,
    dirs: &DirsCliArgs,
) -> eyre::Result<ImportedNetworks> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let networks_dir = data_dir.join(NETWORK_DIRECTORY);
    let network_definitions_path = networks_dir.join(NETWORK_DEFINITIONS_FILE);
    return match load_from_json_file(&network_definitions_path) {
        Ok(networks_map) => Ok(networks_map),
        Err(..) => {
            create_dir_all(&networks_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create network directory: {:?}: {:?}",
                    network_definitions_path, e
                ))
            })?;

            let networks_map = ImportedNetworks::new();
            write_to_json_file(&network_definitions_path, &networks_map, true)
                .map_err(|e| eyre::eyre!(e))?;
            Ok(networks_map)
        }
    };
}

pub fn get_or_create_network_config(
    chain_id: u64,
    address: Address,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<NetworkConfig> {
    let data_dir = dirs.data_dir(Some(chain_id))?;
    let networks_dir = data_dir.join(NETWORK_DIRECTORY);
    let network_config_dir = networks_dir.join(address.to_string());
    let network_config_path = network_config_dir.join(NETWORK_CONFIG_FILE);
    return match load_from_json_file(&network_config_path) {
        Ok(network) => Ok(network),
        Err(..) => {
            create_dir_all(&network_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to create network config directory: {:?}: {:?}",
                    network_config_dir, e
                ))
            })?;

            let network = NetworkConfig::new(address, chain_id, alias);
            write_to_json_file(&network_config_path, &network, true).map_err(|e| eyre::eyre!(e))?;
            Ok(network)
        }
    };
}

pub fn get_network_config(
    chain_id: u64,
    alias: String,
    dirs: &DirsCliArgs,
) -> eyre::Result<NetworkConfig> {
    let network_definitions = get_or_create_network_definitions(chain_id, dirs)?;
    if let Some((_, address)) = network_definitions.get_key_value(&alias) {
        let network_config = get_or_create_network_config(chain_id, *address, alias, dirs)?;
        Ok(network_config)
    } else {
        print_error_message("Network with the provided alias is not imported.");
        Err(eyre::eyre!(""))
    }
}

pub fn set_foundry_signing_method(
    network_config: &NetworkConfig,
    eth: &mut EthereumOpts,
) -> eyre::Result<()> {
    if let Some(signing_method) = network_config.signing_method.clone() {
        match signing_method {
            SigningMethod::Keystore => {
                let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;
                eth.wallet.keystore_path = Some(
                    network_config
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
