use alloy_primitives::Address;
use foundry_common::provider::RetryProvider;
use serde::Deserialize;

use crate::utils::print_loading_until_async;

use super::calls::{is_network, is_opted_in_network};

// Symbiotic network metadata
pub const SYMBIOTIC_GITHUB_URL: &str =
    "https://raw.githubusercontent.com/symbioticfi/metadata-mainnet/refs/heads/main/networks";
pub const SYMBIOTIC_NETWORK_FILE_NAME: &str = "info.json";

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
    let network_address = network_address.to_string().to_lowercase();
    let url = format!("{SYMBIOTIC_GITHUB_URL}/{network_address}/{SYMBIOTIC_NETWORK_FILE_NAME}",);
    let res = reqwest::get(&url).await?;
    let vault_info: Option<NetworkInfo> = serde_json::from_str(&res.text().await?).ok();
    Ok(vault_info)
}

pub async fn validate_network_symbiotic_status<A: TryInto<Address>>(
    network: A,
    network_registry: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_network = print_loading_until_async(
        "Validating network Symbiotic status",
        is_network(network, network_registry, provider),
    )
    .await?;

    if !is_network {
        eyre::bail!("Network is not registered in Symbiotic");
    }

    Ok(())
}

pub async fn validate_network_opt_in_status<A: TryInto<Address>>(
    operator: A,
    network: A,
    opt_in_service: A,
    provider: &RetryProvider,
) -> eyre::Result<()>
where
    A::Error: std::error::Error + Send + Sync + 'static,
{
    let is_opted_in = print_loading_until_async(
        "Checking opted in status",
        is_opted_in_network(operator, network, opt_in_service, &provider),
    )
    .await?;

    if !is_opted_in {
        eyre::bail!("Operator is not opted in network.");
    }

    Ok(())
}
