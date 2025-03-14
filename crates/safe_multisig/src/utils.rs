use std::{
    future::Future,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use alloy_primitives::{Address, U256};
use dialoguer::{theme::ColorfulTheme, Input};

use crate::{transaction_data::SafeTransactionData, SafeMetaTransaction};

#[derive(Debug, thiserror::Error)]
pub enum ExecuteError {
    #[error("User cancelled")]
    UserCancelled,

    #[allow(dead_code)]
    #[error("Ignorable error")]
    Ignore,

    #[error("Other error: {0}")]
    Other(#[from] eyre::Error),
}

pub fn build_safe_tx(tx: SafeMetaTransaction, nonce: U256) -> eyre::Result<SafeTransactionData> {
    Ok(SafeTransactionData {
        to: tx.to.to_checksum(None),
        value: tx.value.try_into()?,
        data: tx.input,
        operation: 0,
        safe_tx_gas: 0,
        base_gas: 0,
        gas_price: 0,
        gas_token: Address::ZERO,
        refund_receiver: Address::ZERO,
        nonce: nonce.try_into()?,
    })
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

pub fn read_user_confirmation() -> eyre::Result<String> {
    Ok(Input::with_theme(&ColorfulTheme::default())
        .validate_with(|input: &String| -> std::result::Result<(), &str> {
            let normalized = input.trim().to_lowercase();
            match normalized.as_str() {
                "y" | "yes" | "n" | "no" => Ok(()),
                _ => Err("Please type 'y/yes' or 'n/no'"),
            }
        })
        .interact()
        .map_err(|e: dialoguer::Error| match e {
            dialoguer::Error::IO(e) => match e.kind() {
                std::io::ErrorKind::Interrupted => ExecuteError::UserCancelled,
                _ => ExecuteError::Other(e.into()),
            },
        })?)
}
