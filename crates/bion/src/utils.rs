use std::{
    fs,
    future::Future,
    io::{BufReader, Write},
    net::IpAddr,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use alloy_chains::Chain;
use alloy_dyn_abi::JsonAbiExt;
use alloy_json_abi::Function;
use alloy_network::{AnyNetwork, TxSigner};
use alloy_primitives::Address;
use alloy_provider::Provider;
use alloy_transport::Transport;
use eyre::ContextCompat;
use foundry_cli::opts::{EthereumOpts, EtherscanOpts, RpcOpts};
use foundry_wallets::WalletSigner;
use futures_util::future::join_all;

use hyve_primitives::alloy_primitives::{hex, U256};
use serde::{de::DeserializeOwned, Serialize};
use tracing::trace;

use crate::common::{DirsCliArgs, NetworkCliArgs};

pub async fn validate_cli_args(address: Option<Address>, eth: &EthereumOpts) -> eyre::Result<()> {
    if let Some(address) = address {
        validate_address_with_signer(address, eth).await?;
    }
    validate_chain_id(&eth.etherscan)?;
    validate_rpc_url(&eth.rpc)?;

    Ok(())
}

pub async fn validate_address_with_signer(
    address: Address,
    eth: &EthereumOpts,
) -> eyre::Result<()> {
    let signer = eth.wallet.signer().await?;
    let from = signer.address();

    match address.to_string().to_lowercase() == from.to_string().to_lowercase() {
        true => Ok(()),
        false => Err(eyre::eyre!("Address does not match signer!")),
    }
}

pub fn validate_rpc_url(rpc: &RpcOpts) -> eyre::Result<()> {
    match rpc.url.is_some() {
        true => Ok(()),
        false => Err(eyre::eyre!("RPC URL is required!")),
    }
}

pub fn validate_chain_id(eth: &EtherscanOpts) -> eyre::Result<()> {
    if let Some(chain_id) = eth.chain {
        match chain_id.id() {
            1 | 17000 | 11155111 => return Ok(()),
            _ => return Err(eyre::eyre!("Invalid ChainID!")),
        }
    }

    Err(eyre::eyre!("ChainID is required!"))
}

/// Clears a specified number of previous lines in the terminal output
///
/// # Arguments
///
/// * `num_lines` - The number of lines to clear, starting from the current cursor position
///
/// # Example
///
/// ```
/// clear_previous_lines(3); // Clears the previous 3 lines of terminal output
/// ```
pub fn clear_previous_lines(num_lines: u16) {
    print!("\x1B[{}F", num_lines); // Move up N lines
    print!("\x1B[J"); // Clear from cursor to end of screen
    print!("\x1B[0G"); // Move to start of line
    std::io::stdout().flush().unwrap();
}

/// Runs a loading animation until a condition is met or a future completes
///
/// # Examples
/// ```
/// // With a boolean condition
/// let done = Arc::new(AtomicBool::new(false));
/// let done_clone = done.clone();
/// print_loading_until("Working...", move || done_clone.load(Ordering::Relaxed));
///
/// // With an async function
/// print_loading_until("Processing...", async_function()).await;
///
/// // With a closure
/// print_loading_until("Calculating...", || some_condition == true);
/// ```
pub fn print_loading_until<F>(message: &str, condition: F)
where
    F: Fn() -> bool,
{
    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut i = 0;

    print!("\r{} {} ", message, spinner[0]);
    std::io::stdout().flush().unwrap();

    while !condition() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        i = (i + 1) % spinner.len();
        print!("\r{} {} ", message, spinner[i]);
        std::io::stdout().flush().unwrap();
    }

    // Clear the line when done
    print!("\r{}\r", " ".repeat(message.len() + 2));
    std::io::stdout().flush().unwrap();
}

/// Async version that runs a loading animation until a future completes
pub async fn print_loading_until_async<F, T>(message: &str, future: F) -> T
where
    F: Future<Output = T>,
{
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let message = message.to_string(); // Clone the message string

    // Spawn the animation in a separate thread
    std::thread::spawn(move || {
        let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut i = 0;

        while running_clone.load(Ordering::Relaxed) {
            print!("\r{} {} ", message, spinner[i]);
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
            i = (i + 1) % spinner.len();
        }

        print!("\r{}\r", " ".repeat(message.len() + 2));
        std::io::stdout().flush().unwrap();
    });

    // Wait for the future to complete
    let result = future.await;

    // Stop the animation
    running.store(false, Ordering::Relaxed);
    std::thread::sleep(std::time::Duration::from_millis(100)); // Give animation thread time to clean up

    result
}

/// Write some object to a file as JSON.
///
/// The file must be created new, it must not already exist.
pub fn write_to_json_file<P: AsRef<Path>, S: Serialize>(
    path: P,
    contents: &S,
    create_new: bool,
) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create_new(create_new)
        .open(&path)
        .map_err(|e| format!("Failed to open {:?}: {:?}", path.as_ref(), e))?;
    serde_json::to_writer(&mut file, contents)
        .map_err(|e| format!("Failed to write JSON to {:?}: {:?}", path.as_ref(), e))
}

/// Load an object from a JSON file.
pub fn load_from_json_file<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> Result<T, String> {
    eprintln!("Loading {:?}", path.as_ref());
    let file = fs::File::open(&path)
        .map_err(|e| format!("Failed to open {:?}: {:?}", path.as_ref(), e))?;
    let reader = BufReader::new(file);

    let data = serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to read JSON from {:?}: {:?}", path.as_ref(), e))?;

    Ok(data)
}

/// Parses a `T` from a string using [`serde_json::from_str`].
pub fn parse_json<T: DeserializeOwned>(value: &str) -> serde_json::Result<T> {
    serde_json::from_str(value)
}
