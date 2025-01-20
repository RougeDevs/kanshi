use std::env;
use clap::{Arg, Command};
use starknet::core::types::Felt;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub storage_url: String,
    pub apibara_key: String,
    pub network: NetworkName,
    pub contract_address: Felt,
    // pub filter: String,
    pub starting_block: u64,
    pub write_path: String
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkName {
    Mainnet,
    Sepolia,
}

impl NetworkName {
    fn from_str(input: &str) -> Result<Self, String> {
        match input.to_lowercase().as_str() {
            "mainnet" => Ok(NetworkName::Mainnet),
            "sepolia" => Ok(NetworkName::Sepolia),
            _ => Err(format!("Invalid network name: {}", input)),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        dotenv::dotenv().ok();

        // Parse CLI arguments
        let matches = Command::new("Welcome to Kanshi!!")
            .version("1.0")
            .author("RougeDevs")
            .about("Configures and runs the application")
            .arg(
                Arg::new("redis-url")
                    .long("redis-url")
                    .value_name("REDIS_URL")
                    .help("Sets the Redis URL")
                    .num_args(1),
            )
            .arg(
                Arg::new("apibara-key")
                    .long("apibara-key")
                    .value_name("APIBARA_KEY")
                    .help("Sets the Apibara API key")
                    .num_args(1),
            )
            .arg(
                Arg::new("starting-block")
                    .long("starting-block")
                    .value_name("STARTING_BLOCK")
                    .help("Sets the starting block")
                    .num_args(1),
            )
            .arg(
                Arg::new("contract-address")
                .long("contract-address")
                .value_name("CONTRACT_ADDRESS")
                .help("Set contract address to listen to")
                .num_args(1)
            )
            // .arg(
            //     Arg::new("filter")
            //     .long("filter")
            //     .value_name("filter")
            //     .help("Set event filters")
            //     .num_args(10)
            // )
            .arg(
                Arg::new("network")
                    .long("network")
                    .value_name("NETWORK")
                    .help("Sets the network (Mainnet or Sepolia)")
                    .num_args(1),
            )
            .get_matches();

        Ok(Config {
            storage_url: matches
                .get_one::<String>("redis-url")
                .cloned()
                .unwrap_or_else(|| env::var("REDIS_URL").unwrap_or_else(|_| "redis://123.0.0.1:6379".to_string())),
            apibara_key: matches
                .get_one::<String>("apibara-key")
                .cloned()
                .unwrap_or_else(|| env::var("APIBARA_KEY").expect("Missing APIBARA_KEY")),
            network: matches
                .get_one::<String>("network")
                .map(|v| NetworkName::from_str(v).expect("Invalid network value"))
                .unwrap_or(NetworkName::Mainnet),
            contract_address: Felt::from_hex(&env::var("CONTRACT_ADDRESS").expect("Missing CONTRACT_ADDRESS"))?,
            starting_block: matches
                .get_one::<String>("starting-block")
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(|| {
                    env::var("STARTING_BLOCK")
                        .unwrap_or_else(|_| "0".to_string())
                        .parse()
                        .expect("STARTING_BLOCK must be a valid number")
                }),
            write_path: env::var("WRITE_PATH").unwrap_or_else(|_| "indexer_state.json".to_string())
        })
    }
}
