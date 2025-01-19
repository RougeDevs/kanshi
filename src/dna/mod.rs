use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use crate::config::{Config, NetworkName};
use tokio::sync::mpsc;
use crate::services::dataStore::StorageManager;
use crate::utils::conversions::felt_as_apibara_field;
use anyhow::Result;
use apibara_core::starknet::v1alpha2::Event;
use apibara_core::{
    node::v1alpha2::DataFinality,
    starknet::v1alpha2::{Block, Filter, HeaderFilter},
};
use apibara_sdk::{configuration, ClientBuilder, Configuration, Uri};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

const INDEXING_STREAM_CHUNK_SIZE: usize = 32;

#[derive(Serialize, Deserialize)]
struct BlockState {
    last_processed_block: u64,
}

#[derive(Clone)]
pub struct IndexerService {
    config: Config,
    uri: Uri,
    stream_config: Configuration<Filter>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventData {
    pub block_number: u64,
    pub from_address: String,
    pub timestamp: u64,
    pub transaction_hash: String,
    pub data: Vec<String>,
}

impl IndexerService {
    async fn save_block_state(&self, block_number: u64) -> Result<()> {
        let state = BlockState {
            last_processed_block: block_number,
        };
        
        let state_path = self.get_state_file_path();
        let state_json = serde_json::to_string(&state)?;
        fs::write(state_path, state_json)?;
        
        Ok(())
    }

    fn load_block_state(&self) -> Result<Option<u64>> {
        let state_path = self.get_state_file_path();
        
        if !state_path.exists() {
            return Ok(None);
        }
        
        let state_json = fs::read_to_string(state_path)?;
        let state: BlockState = serde_json::from_str(&state_json)?;
        
        Ok(Some(state.last_processed_block))
    }

    // Helper to get state file path
    fn get_state_file_path(&self) -> PathBuf {
        PathBuf::from("indexer_state.json")
    }


    pub async fn new(config: Config) -> Self {
        // First create with default starting block
        let uri = match config.network {
            NetworkName::Mainnet => Uri::from_static("https://mainnet.starknet.a5a.ch"),
            NetworkName::Sepolia => Uri::from_static("https://sepolia.starknet.a5a.ch"),
        };

        // Create initial service with config's starting block
        let mut service = IndexerService {
            config: config.clone(),
            uri,
            stream_config: Configuration::<Filter>::default()
                .with_starting_block(config.starting_block)
                .with_finality(DataFinality::DataStatusPending)
                .with_filter(|mut filter| {
                    filter
                        .with_header(HeaderFilter::weak())
                        .add_event(|event| {
                            event.with_from_address(felt_as_apibara_field(&config.contract_address))
                        })
                        .build()
                }),
        };

        // Try to load saved block state
        if let Ok(Some(block_number)) = service.load_block_state() {
            // Update stream_config with loaded block number
            service.stream_config = Configuration::<Filter>::default()
                .with_starting_block(block_number)
                .with_finality(DataFinality::DataStatusPending)
                .with_filter(|mut filter| {
                    filter
                        .with_header(HeaderFilter::weak())
                        .add_event(|event| {
                            event.with_from_address(felt_as_apibara_field(&config.contract_address))
                        })
                        .build()
                });
            println!("✅ [Indexer] Loaded last processed block: {}", block_number);
        } else {
            println!("✅ [Indexer] Starting from initial block: {}", config.starting_block);
        }

        service
    }

    pub async fn run_forever_simplified(&mut self, tx: &mpsc::UnboundedSender<Event>) -> Result<()> {
        println!("✅ [Indexer] Starting event listener...");
        let mut reached_pending_block: bool = false;
        let (config_client, config_stream) = configuration::channel(INDEXING_STREAM_CHUNK_SIZE);
        
        // Initialize the stream with configuration
        config_client.send(self.stream_config.clone()).await?;

        let mut stream = ClientBuilder::default()
            .with_bearer_token(Some(self.config.apibara_key.clone()))
            .connect(self.uri.clone())
            .await
            .unwrap()
            .start_stream::<Filter, Block, _>(config_stream)
            .await
            .unwrap();
        
        println!("✅ [Indexer] Connected to Apibara, listening ...");

        loop {
            match stream.try_next().await {
                Ok(Some(response)) => {
                    match response {
                        apibara_sdk::DataMessage::Data {
                            cursor: _,
                            end_cursor: _,
                            finality,
                            batch,
                        } => {
                            if finality == DataFinality::DataStatusPending && !reached_pending_block {
                                println!("[🔍 Indexer] 🥳🎉 Reached pending block!");
                                reached_pending_block = true;
                            }
                            
                            for block in batch {
                                let block_number = block.header.as_ref()
                                    .map(|hdr| hdr.block_number)
                                    .unwrap_or(0);
                                for event in block.events {
                                    if let Some(event) = event.event {
                                        let block_number = block.header.as_ref()
                                            .map(|hdr| hdr.block_number)
                                            .unwrap_or(0);
                                        
                                        println!("\n\n📦 [APIBARA EVENT RECEIVED] Block: {}\n\n", block_number);
        
                                        if tx.send(event).is_err() {
                                            println!("⚠️ [Warning] Receiver dropped, stopping indexer...");
                                            return Ok(());
                                        }
                                    }
                                }
                                if let Err(e) = self.save_block_state(block_number).await {
                                    println!("⚠️ [Warning] Failed to save block state: {:?}", e);
                                }
                            }
                        }
                        apibara_sdk::DataMessage::Invalidate { cursor } => {
                            if let Some(c) = cursor {
                                return Err(anyhow::anyhow!(
                                    "Received an invalidate request data at {}",
                                    &c.order_key
                                ));
                            }
                            return Err(anyhow::anyhow!("Invalidate request without cursor provided"));
                        }
                        apibara_sdk::DataMessage::Heartbeat => {
                            println!("❤️ Heartbeat received");
                        }
                    }
                },
                Ok(None) => {
                    println!("Stream ended");
                    break;
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("Error while streaming: {:?}", e));
                }
            }
        }
        Ok(())
    }

}