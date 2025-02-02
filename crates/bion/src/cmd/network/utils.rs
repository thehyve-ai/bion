use alloy_primitives::Address;
use serde::Deserialize;

use std::fs::create_dir_all;

use crate::{
    common::DirsCliArgs,
    utils::{load_from_json_file, write_to_json_file},
};

use super::{
    consts::{
        NETWORK_CONFIG_FILE, NETWORK_DEFINITIONS_FILE, NETWORK_DIRECTORY, NETWORK_FILE_NAME,
        SYMBIOTIC_GITHUB_URL,
    },
    ImportedNetworks, NetworkConfig,
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
    let url = format!("{SYMBIOTIC_GITHUB_URL}/{network_address}/{NETWORK_FILE_NAME}",);
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

            let network = NetworkConfig::new(address, alias);
            write_to_json_file(&network_config_path, &network, true).map_err(|e| eyre::eyre!(e))?;
            Ok(network)
        }
    };
}
