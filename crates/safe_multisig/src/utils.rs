use std::{
    future::Future,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;

use crate::transaction_data::SafeTransactionData;

pub fn build_safe_tx(
    data: Bytes,
    tx: WithOtherFields<TransactionRequest>,
    nonce: U256,
) -> eyre::Result<SafeTransactionData> {
    Ok(SafeTransactionData {
        to: match tx.to.unwrap() {
            TxKind::Call(a) => a,
            _ => {
                eyre::bail!("Invalid tx kind")
            }
        },
        value: tx.value.unwrap_or_else(|| U256::from(0)),
        data,
        operation: 0,
        safe_tx_gas: U256::from(tx.gas.unwrap_or(0)),
        base_gas: U256::from(0),
        gas_price: U256::from(0),
        gas_token: Address::ZERO,
        refund_receiver: Address::ZERO,
        nonce,
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
