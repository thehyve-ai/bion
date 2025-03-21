// This file is derived from the "foundry-cast" crate found in the Foundry project by Paradigm.
// Original source: https://github.com/foundry-rs/foundry/blob/f94ce465969c4b25df57aa3e72ed80d4f6d4599f/crates/cast/bin/cmd/rpc.rs
// License: MIT (see the LICENSE file in crate's root directory)
// Modifications by @KristianRadkov @electricddev, 2024

use ::cast::Cast;
use clap::Parser;
use eyre::Result;
use foundry_cli::{opts::RpcOpts, utils, utils::LoadConfig};
use foundry_common::sh_println;
use itertools::Itertools;

/// CLI arguments for `cast rpc`.
#[derive(Clone, Debug, Parser)]
pub struct RpcArgs {
    /// RPC method name
    method: String,

    /// RPC parameters
    ///
    /// Interpreted as JSON:
    ///
    /// cast rpc eth_getBlockByNumber 0x123 false
    /// => {"method": "eth_getBlockByNumber", "params": ["0x123", false] ... }
    params: Vec<String>,

    /// Send raw JSON parameters
    ///
    /// The first param will be interpreted as a raw JSON array of params.
    /// If no params are given, stdin will be used. For example:
    ///
    /// cast rpc eth_getBlockByNumber '["0x123", false]' --raw
    ///     => {"method": "eth_getBlockByNumber", "params": ["0x123", false] ... }
    #[arg(long, short = 'w')]
    raw: bool,

    #[command(flatten)]
    rpc: RpcOpts,
}

impl RpcArgs {
    #[allow(dead_code)]
    pub async fn run(self) -> Result<()> {
        let Self { raw, method, params, rpc } = self;

        let config = rpc.load_config()?;
        let provider = utils::get_provider(&config)?;

        let params = if raw {
            if params.is_empty() {
                serde_json::Deserializer::from_reader(std::io::stdin())
                    .into_iter()
                    .next()
                    .transpose()?
                    .ok_or_else(|| eyre::format_err!("Empty JSON parameters"))?
            } else {
                value_or_string(params.into_iter().join(" "))
            }
        } else {
            serde_json::Value::Array(params.into_iter().map(value_or_string).collect())
        };
        sh_println!("{}", Cast::new(provider).rpc(&method, params).await?)?;
        Ok(())
    }
}

#[allow(dead_code)]
fn value_or_string(value: String) -> serde_json::Value {
    serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value))
}
