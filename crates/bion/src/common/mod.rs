use clap::{value_parser, Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use consts::{DEFAULT_NETWORK_DIR, DEFAULT_OPERATOR_DIR, DEFAULT_ROOT_DIR};

pub mod consts;

#[derive(ValueEnum, Clone, Debug)]
pub enum Networks {
    #[value(alias("sepolia"))]
    Sepolia,
    #[value(alias("mainnet"))]
    Mainnet,
}

impl Networks {
    pub fn as_str(&self) -> &str {
        match self {
            Networks::Mainnet => "mainnet",
            Networks::Sepolia => "sepolia",
        }
    }

    pub fn get_by_chain_id(chain_id: u64) -> eyre::Result<String> {
        match chain_id {
            1 => Ok(Networks::Mainnet.as_str().to_string()),
            11155111 => Ok(Networks::Sepolia.as_str().to_string()),
            _ => Err(eyre::eyre!("Chain ID not supported")),
        }
    }
}

#[derive(Debug, Parser, Clone)]
pub struct DirsCliArgs {
    #[arg(
        long,
        required = false,
        value_name = "OPERATORS_DIR",
        conflicts_with = "data_dir",
        help = "The directory which contains the operator keystores and data. \
                    Defaults to {data_dir_loc}/{network}/operators."
    )]
    operators_dir: Option<PathBuf>,

    #[clap(flatten)]
    data_dir: DatadirCliArgs,
}

impl DirsCliArgs {
    pub fn data_dir(&self, chain_id: Option<u64>) -> eyre::Result<PathBuf> {
        let network = if let Some(chain_id) = chain_id {
            Networks::get_by_chain_id(chain_id)?
        } else {
            Networks::Mainnet.as_str().to_string()
        };
        let data_dir = self.data_dir.get_data_dir().join(network);
        Ok(data_dir)
    }

    pub fn operators_dir(&self, chain_id: Option<u64>) -> eyre::Result<PathBuf> {
        let data_dir = self.data_dir(chain_id)?;
        let operators_dir =
            self.operators_dir.clone().unwrap_or_else(|| data_dir.join(DEFAULT_OPERATOR_DIR));
        Ok(operators_dir)
    }
}

#[derive(Debug, Parser, Clone)]
pub struct DatadirCliArgs {
    #[arg(
        long,
        required = false,
        value_parser = value_parser!(PathBuf),
        help = "Used to specify a custom root data directory for hyve keys and databases. \
                    Defaults to home_dir/.hyve if the home dir is available, otherwise it defaults to `.` \
                    Note: Users should specify separate custom datadirs for different networks."
    )]
    data_dir: Option<PathBuf>,
}

impl DatadirCliArgs {
    pub fn get_data_dir(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(DEFAULT_ROOT_DIR))
                .unwrap_or_else(|| PathBuf::from("."))
        })
    }
}

#[derive(Debug, Parser)]
pub struct NetworkCliArgs {
    #[arg(
        long,
        value_name = "DIR",
        help = "Data directory for network keys. Defaults to ~/.hyve/network"
    )]
    network_dir: Option<String>,

    #[arg(long, default_value = "9002", help = "The port to listen on for the libp2p beacon node")]
    pub quic_port: u16,

    #[arg(
        long,
        value_name = "ADDRESS",
        default_value = "0.0.0.0",
        help = "The address hyve will listen for UDP connections. Currently only IpV4 is supported."
    )]
    pub listen_address: String,

    #[arg(
        long,
        default_value = "false",
        help = "Disables UPnP support. Setting this will prevent Hyve from attempting to automatically establish external port mappings."
    )]
    pub disable_upnp: bool,

    #[arg(
        long,
        help = "One or more comma-delimited multiaddrs to bootstrap the p2p network.",
        allow_hyphen_values = true
    )]
    pub boot_nodes: Option<Vec<String>>,
}

impl NetworkCliArgs {
    pub fn network_dir(&self, datadir: PathBuf) -> PathBuf {
        if let Some(dir) = self.network_dir.as_ref() {
            PathBuf::from(dir)
        } else {
            datadir.join(DEFAULT_NETWORK_DIR)
        }
    }
}

#[derive(Clone, Debug, Parser)]
pub struct MetricsCliArgs {
    #[arg(
        long,
        default_value = "9003",
        help = "The port used for exposing Prometheus metrics. \
                    \
                    Note - when running an RPC server, the RPC port is used instead.",
        conflicts_with = "rpc_port"
    )]
    pub metrics_port: u16,

    #[arg(
        long,
        default_value = "20s",
        help = "Sets the bucket width when using summaries. \
                    Summaries are rolling, which means that they are divided into buckets of a fixed duration (width), and older buckets are dropped as they age out. This means data from a period as large as the width will be dropped at a time. \
                    The total amount of data kept for a summary is the number of buckets times the bucket width. For example, a bucket count of 3 and a bucket width of 20 seconds would mean that 60 seconds of data is kept at most, with the oldest 20 second chunk of data being dropped as the summary rolls forward. \
                    Use more buckets with a smaller width to roll off smaller amounts of data at a time, or fewer buckets with a larger width to roll it off in larger chunks."
    )]
    pub bucket_duration: humantime::Duration,

    #[arg(
        long,
        default_value = "3",
        help = "Sets the bucket count when using summaries. \
                    Summaries are rolling, which means that they are divided into buckets of a fixed duration (width), and older buckets are dropped as they age out. This means data from a period as large as the width will be dropped at a time. \
                    The total amount of data kept for a summary is the number of buckets times the bucket width. For example, a bucket count of 3 and a bucket width of 20 seconds would mean that 60 seconds of data is kept at most, with the oldest 20 second chunk of data being dropped as the summary rolls forward. \
                    Use more buckets with a smaller width to roll off smaller amounts of data at a time, or fewer buckets with a larger width to roll it off in larger chunks."
    )]
    pub bucket_count: u32,

    #[arg(
        long,
        default_value = "5s",
        help = "The idle timeout for metrics. \
                    If a metric hasn't been updated within this timeout, it will be removed from the registry and in turn removed \
                    from the normal scrape output until the metric is emitted again.  This behavior is driven by requests to \
                    generate rendered output, and so metrics will not be removed unless a request has been made recently enough to \
                    prune the idle metrics."
    )]
    pub idle_timeout: humantime::Duration,

    #[arg(
        long,
        default_value = "5s",
        help = "Sets the upkeep interval. \
                    The upkeep task handles periodic maintenance operations, such as draining histogram data, to ensure that all recorded data is up-to-date and prevent unbounded memory growth."
    )]
    pub upkeep_timeout: humantime::Duration,

    #[arg(
        long,
        value_parser = clap::builder::ValueParser::new(parse_key_value),
        help = "Adds a global label to this exporter. \
                    Global labels are applied to all metrics. Labels defined on the metric key itself have precedence over any global labels. If this method is called multiple times, the latest value for a given label key will be used."
    )]
    pub global_label: Option<(String, String)>,

    #[arg(
        long,
        default_value = "1s",
        help = "The interval at which process metrics are collected."
    )]
    pub process_metrics_interval: humantime::Duration,
}

#[derive(Clone, Debug, Parser)]
pub struct EncodingValidationCliArgs {
    #[arg(long, default_value = "10", help = "The number of encoding validations per thread.")]
    pub capacity_per_thread: usize,

    #[arg(long, default_value = "8", help = "The number of threads for the encoding validator.")]
    pub num_threads: usize,

    #[arg(long, default_value = "60", help = "The timeout for encoding transactions.")]
    pub transaction_timeout: u64,
}

#[derive(Clone, Debug, Parser)]
pub struct SlotClockCliArgs {
    #[arg(long, default_value = "0", help = "The genesis slot of the chain.")]
    pub genesis_slot: u64,

    #[arg(long, default_value = "1606824023", help = "The genesis time of the chain.")]
    pub genesis_timestamp: u64,

    #[arg(long, default_value = "12", help = "The number of seconds per slot.")]
    pub seconds_per_slot: u64,
}

/// Parse a single key-value pair
fn parse_key_value(env: &str) -> Result<(String, String), String> {
    if let Some((var, value)) = env.split_once('=') {
        Ok((var.to_owned(), value.to_owned()))
    } else {
        Err("invalid key-value pair".into())
    }
}

#[derive(Debug, Parser)]
pub struct KeystoreCliArgs {
    #[arg(long, required = true, help = "The password that will be used to unlock the keystore.")]
    keystore_password: String,

    #[arg(long, required = true, help = "The path to the keystore file.")]
    keystore_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SigningMethod {
    Keystore,
    Ledger,
    Trezor,
    MultiSig,
}
