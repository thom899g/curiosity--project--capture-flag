use std::sync::Arc;
use ethers::prelude::*;
use ethers::providers::Ws;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, debug};
use crate::config::MempoolFilters;

pub async fn start_listener(
    provider: Provider<Ws>,
    tx: mpsc::Sender<Transaction>,
    filters: MempoolFilters,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("👂 Starting mempool listener...");
    
    let provider = Arc::new(provider);
    
    // Subscribe to pending transactions
    let mut stream = match provider.subscribe_pending_txs().await {
        Ok(stream) => {
            info!("✅ Subscribed to pending transactions");
            stream
        }
        Err(e) => {
            error!("❌ Failed to subscribe to pending transactions: {}", e);
            return Err(e.into());
        }
    };
    
    // Process incoming transactions
    while let Some(tx_hash) = stream.next().await {
        debug!("📥 Received transaction: {}", tx_hash);
        
        // Fetch full transaction details
        match provider.get_transaction(tx_hash).await {
            Ok(Some(tx)) => {
                if should_process_tx(&tx, &filters) {
                    debug!("✅ Transaction passed filters: {}", tx.hash);
                    
                    // Send to channel for further processing
                    if tx.send(tx.clone()).await.is_err() {
                        warn!("Channel closed, stopping listener");
                        break;
                    }
                }
            }
            Ok(None) => {
                debug!("Transaction not found: {}", tx_hash);
            }
            Err(e) => {
                warn!("Error fetching transaction {}: {}", tx_hash, e);
            }
        }
    }
    
    Ok(())
}

fn should_process_tx(tx: &Transaction, filters: &MempoolFilters) -> bool {
    // Check gas price limits
    if let Some(max_gas_price) = filters.max_gas_price {
        if let Some(gas_price) = tx.gas_price {
            if gas_price.as_u64() > max_gas_price {
                return false;
            }
        }
    }
    
    if let Some(min_gas_price) = filters.min_gas_price {
        if let Some(gas_price) = tx.gas_price {
            if gas_price.as_u64() < min_gas_price {
                return false;
            }
        }
    }
    
    // Check if sender is blocked
    if let Some(from) = tx.from {
        if filters.blocked_senders.contains