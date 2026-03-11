use std::env;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    // Network configuration
    pub chain_id: u64,
    pub blocknative_ws_url: String,
    pub alchemy_http_url: String,
    pub flashbots_rpc_url: String,
    pub tenderly_api_url: String,
    
    // Wallet configuration
    pub private_key: String,
    pub executor_address: String,
    pub target_vault_address: String,
    
    // Trading parameters
    pub min_profit_wei: u128,
    pub max_gas_price_gwei: u64,
    pub max_slippage_bps: u16,
    pub max_daily_loss_wei: u128,
    
    // DEX addresses
    pub dex_addresses: HashMap<String, String>,
    
    // Mempool filters
    pub mempool_filters: MempoolFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolFilters {
    pub min_gas_price: Option<u64>,
    pub max_gas_price: Option<u64>,
    pub allowed_senders: Vec<String>,
    pub blocked_senders: Vec<String>,
    pub method_ids: Vec<String>,
}

impl Config {
    pub fn load_config() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();
        
        Ok(Config {
            chain_id: env::var("CHAIN_ID")
                .map(|s| s.parse().map_err(|_| ConfigError::InvalidConfig("Invalid chain ID".to_string())))
                .unwrap_or(Ok(1))?,
            
            blocknative_ws_url: env::var("BLOCKNATIVE_WS_URL")
                .unwrap_or_else(|_| "wss://api.blocknative.com/v0".to_string()),
            
            alchemy_http_url: env::var("ALCHEMY_HTTP_URL")
                .ok_or_else(|| ConfigError::MissingEnv("ALCHEMY_HTTP_URL".to_string()))?,
            
            flashbots_rpc_url: env::var("FLASHBOTS_RPC_URL")
                .unwrap_or_else(|_| "https://relay.flashbots.net".to_string()),
            
            tenderly_api_url: env::var("TENDERLY_API_URL")
                .unwrap_or_else(|_| "https://api.tenderly.co/api/v1".to_string()),
            
            private_key: env::var("EXECUTOR_PRIVATE_KEY")
                .ok_or_else(|| ConfigError::MissingEnv("EXECUTOR_PRIVATE_KEY".to_string()))?,
            
            executor_address: env::var("EXECUTOR_ADDRESS")
                .ok_or_else(|| ConfigError::MissingEnv("EXECUTOR_ADDRESS".to_string()))?,
            
            target_vault_address: env::var("TARGET_VAULT_ADDRESS")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()),
            
            min_profit_wei: env::var("MIN_PROFIT_WEI")
                .map(|s| s.parse().unwrap_or(100000000000000000)) // 0.1 ETH default
                .unwrap_or(100000000000000000),
            
            max_gas_price_gwei: env::var("MAX_GAS_PRICE_GWEI")
                .map(|s| s.parse().unwrap_or(200))
                .unwrap_or(200),
            
            max_slippage_bps: env::var("MAX_SLIPPAGE_BPS")
                .map(|s| s.parse().unwrap_or(200)) // 2%
                .unwrap_or(200),
            
            max_daily_loss_wei: env::var("MAX_DAILY_LOSS_WEI")
                .map(|s| s.parse().unwrap_or(500000000000000000)) // 0.5 ETH default
                .unwrap_or(500000000000000000),
            
            dex_addresses: Self::load_dex_addresses(),
            
            mempool_filters: MempoolFilters {
                min_gas_price: None,
                max_gas_price: Some(200),
                allowed_senders: vec![],
                blocked_senders: vec![],
                method_ids: vec![
                    "0x7ff36ab5".to_string(), // swapExactETHForTokens
                    "0x18cbafe5".to_string(), // swapExactTokensForETH
                    "0x791ac947".to_string(), // swapExactTokensForTokens
                ],
            },
        })
    }
    
    fn load_dex_addresses() -> HashMap<String, String> {
        let mut addresses = HashMap::new();
        
        // Uniswap V2
        addresses.insert("uniswap_v2_router".to_string(), 
            env::var("UNISWAP_V2_ROUTER").unwrap_or_else(|_| 
                "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".to_string()));
        
        addresses.insert("uniswap_v2_factory".to_string(),
            env::var("UNISWAP_V2_FACTORY").unwrap_or_else(|_|
                "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".to_string()));
        
        // Uniswap V3
        addresses.insert("uniswap_v3_router".to_string(),
            env::var("UNISWAP_V3_ROUTER").unwrap_or_else(|_|
                "0xE592427A0AEce92De3Edee1F18E0157C05861564".to_string()));
        
        // Sushiswap
        addresses.insert("sushiswap_router".to_string(),
            env::var("SUSHISWAP_ROUTER").unwrap_or_else(|_|
                "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F".to_string()));
        
        addresses
    }
}