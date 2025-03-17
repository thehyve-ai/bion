use ctor::ctor;
use std::env;
use std::path::Path;

#[ctor]
fn init() {
    let home = env::var("HOME").expect("HOME env var not set");
    let foundry_bin = format!("{}/.foundry/bin", home);

    if !Path::new(&foundry_bin).join("anvil").exists() {
        panic!(
            "Anvil binary not found in {}. Please check your Foundry installation.",
            foundry_bin
        );
    }

    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", foundry_bin, current_path);
    env::set_var("PATH", new_path);
}
