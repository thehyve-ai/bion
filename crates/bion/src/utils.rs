use account_utils::{helpers::random_password_string, ZeroizeString};
use alloy_network::TxSigner;
use alloy_primitives::Address;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use foundry_cli::opts::{EthereumOpts, RpcOpts};
use serde::{de::DeserializeOwned, Serialize};

use std::{
    fs,
    future::Future,
    io::{BufReader, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ExecuteError {
    #[error("User cancelled")]
    UserCancelled,

    #[error("Ignorable error")]
    Ignore,

    #[error("Other error: {0}")]
    Other(#[from] eyre::Error),
}

pub fn validate_cli_args(eth: &EthereumOpts) -> eyre::Result<()> {
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
        false => Err(eyre::eyre!(
            "Address does not match signer! Address: {}, Signer: {}",
            address,
            from
        )),
    }
}

pub fn validate_rpc_url(rpc: &RpcOpts) -> eyre::Result<()> {
    match rpc.url.is_some() {
        true => Ok(()),
        false => {
            print_error_message("RPC URL is required!");
            eyre::bail!("")
        }
    }
}

pub fn print_success_message(msg: &str) {
    println!("{}", msg.green());
}

pub fn print_error_message(msg: &str) {
    println!("{}", msg.bold().red());
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

pub fn get_keystore_password() -> eyre::Result<ZeroizeString> {
    let options = vec![
        "Enter a custom password",
        "Generate a random strong password",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("\nChoose a password option for the keystore")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| eyre::eyre!(format!("Failed to show password selection menu: {}", e)))?;

    match selection {
        0 => {
            println!("{}", "Please enter a password when prompted.".bright_cyan());
            Ok(get_keystore_password_from_input()?)
        }
        2 => Ok(get_random_keystore_password()?),
        _ => unreachable!(),
    }
}

pub fn get_keystore_password_from_input() -> eyre::Result<ZeroizeString> {
    let password = loop {
        let password = rpassword::prompt_password_stderr("Enter your password:")
            .map_err(|e| format!("Error reading from stdin: {}", e))
            .map(ZeroizeString::from)
            .map_err(|e| eyre::eyre!(e))?;

        let confirmation = rpassword::prompt_password_stderr("Confirm your password:")
            .map_err(|e| eyre::eyre!("Error reading from stdin: {}", e))?;

        if password.as_str() != confirmation {
            clear_previous_lines(2);
            println!(
                "\n{}",
                "❌ Passwords do not match. Please try again.".bright_red()
            );
            continue;
        }
        if password.as_str().trim().is_empty() {
            clear_previous_lines(2);
            println!(
                "\n{}",
                "❌ Password cannot be empty. Please try again.".bright_red()
            );

            continue;
        }
        clear_previous_lines(3);
        break password;
    };

    Ok(password)
}

fn get_random_keystore_password() -> eyre::Result<ZeroizeString> {
    let password = random_password_string();

    println!(
        "\n⚠️  {}",
        "WARNING: Please store this password safely. It will not be shown again!"
            .bright_yellow()
            .bold()
    );
    println!("{}", "Your password is:".bright_white());
    println!("{}", "─".repeat(20));
    println!("{}", password.as_str().bright_yellow().bold());
    println!("{}", "─".repeat(20));
    println!(
        "{}",
        "Please type 'yes' after you have safely stored this password:".bright_cyan()
    );

    loop {
        let confirmation: String = read_user_confirmation()?;

        if confirmation.trim() == "yes" {
            println!("\n{}", "✅ Password confirmed as backed up.".bright_green());
            break;
        } else {
            println!("{}", "Password confirmation cancelled".bright_cyan());
            return Err(eyre::eyre!(ExecuteError::Ignore));
        }
    }
    Ok(password)
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
