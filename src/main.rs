use apibara_core::starknet::v1alpha2::Event;
use config::Config;
use dna::IndexerService;
use tokio::{sync::mpsc, task};

mod dna;
mod config;
mod utils;
mod services;

#[tokio::main]
async fn main() {
    print_banner();
    
    // Create a channel for event communication
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
    
    // Load configurations
    let config = match Config::new() {
        Ok(config) => {
            println!("Configurations loaded ✓");
            config
        }
        Err(e) => {
            eprintln!("Failed to load configuration ❗️ {}", e);
            return;
        }
    };

    // Create the IndexerService instance
    let mut service = IndexerService::new(config);
    
    // Spawn the indexer service in a separate task
    let indexer_handle = task::spawn(async move {
        if let Err(e) = service.await.run_forever_simplified(&tx).await {
            eprintln!("Error running Indexer ❗️ {:#}", e);
        }
    });

    // Spawn the event consumer in a separate task
    let consumer_handle = task::spawn(async move {
        while let Some(event) = rx.recv().await {
            println!("🔥 Received Event: {:?}\n\n", event);
            // Add your event processing logic here
            // For example:
            process_event(event).await;
        }
    });

    // Wait for both tasks to complete
    tokio::select! {
        _ = indexer_handle => println!("Indexer task completed"),
        _ = consumer_handle => println!("Consumer task completed"),
    }
}

async fn process_event(event: Event) {
    // Add your event processing logic here
    // For example:
    match event {
        // Add pattern matching for different event types
        _ => {
            // Default processing
            println!("Processing event: {:?}", event);
        }
    }
}

fn print_banner() {
    println!(
        r#"
 █████   ███   █████          ████                                                 █████            
░░███   ░███  ░░███          ░░███                                                ░░███             
 ░███   ░███   ░███   ██████  ░███   ██████   ██████  █████████████    ██████     ███████    ██████ 
 ░███   ░███   ░███  ███░░███ ░███  ███░░███ ███░░███░░███░░███░░███  ███░░███   ░░░███░    ███░░███
 ░░███  █████  ███  ░███████  ░███ ░███ ░░░ ░███ ░███ ░███ ░███ ░███ ░███████      ░███    ░███ ░███
  ░░░█████░█████░   ░███░░░   ░███ ░███  ███░███ ░███ ░███ ░███ ░███ ░███░░░       ░███ ███░███ ░███
    ░░███ ░░███     ░░██████  █████░░██████ ░░██████  █████░███ █████░░██████      ░░█████ ░░██████ 
     ░░░   ░░░       ░░░░░░  ░░░░░  ░░░░░░   ░░░░░░  ░░░░░ ░░░ ░░░░░  ░░░░░░        ░░░░░   ░░░░░░  
                                                                                                    
                                                                                                    
                                                                                                    
 █████   ████   █████████   ██████   █████  █████████  █████   █████ █████                          
░░███   ███░   ███░░░░░███ ░░██████ ░░███  ███░░░░░███░░███   ░░███ ░░███                           
 ░███  ███    ░███    ░███  ░███░███ ░███ ░███    ░░░  ░███    ░███  ░███                           
 ░███████     ░███████████  ░███░░███░███ ░░█████████  ░███████████  ░███                           
 ░███░░███    ░███░░░░░███  ░███ ░░██████  ░░░░░░░░███ ░███░░░░░███  ░███                           
 ░███ ░░███   ░███    ░███  ░███  ░░█████  ███    ░███ ░███    ░███  ░███                           
 █████ ░░████ █████   █████ █████  ░░█████░░█████████  █████   █████ █████                          
░░░░░   ░░░░ ░░░░░   ░░░░░ ░░░░░    ░░░░░  ░░░░░░░░░  ░░░░░   ░░░░░ ░░░░░                           
                                                                                                    
                                                                                                    
                                                                                                    
"#
    );
}