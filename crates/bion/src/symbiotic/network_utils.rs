use alloy_primitives::Address;
use serde::Deserialize;

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
