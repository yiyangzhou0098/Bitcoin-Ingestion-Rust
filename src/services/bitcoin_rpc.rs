use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::sync::Arc;
use bitcoincore_rpc_json::EstimateMode; // Correct import for EstimateMode
use std::error::Error;
use chrono::{NaiveDateTime, NaiveDate};


pub struct BitcoinRpcService {
    rpc_client: Client,
}

#[derive(Debug)]
pub struct DailyTxData {
    pub date: NaiveDate,
    pub tx_count: usize,
}

impl BitcoinRpcService {
    pub fn new(rpc_url: &str, rpc_user: &str, rpc_password: &str) -> Arc<Self> {
        let rpc_client = Client::new(
            rpc_url,
            Auth::UserPass(rpc_user.to_string(), rpc_password.to_string())
        ).expect("Failed to create Bitcoin RPC client");
        Arc::new(Self { rpc_client })
    }

    pub fn get_block_height(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let block_height = self.rpc_client.get_block_count()?;
        Ok(block_height)
    }

    pub async fn get_last_7_days_tx_data(&self) -> Result<Vec<DailyTxData>, Box<dyn Error>> {
        let mut daily_tx_counts = Vec::new();
        let current_block = match self.rpc_client.get_block_count() {
            Ok(block_count) => block_count,
            Err(e) => {
                eprintln!("Error retrieving block count: {:?}", e);
                return Err(Box::new(e));
            }
        }; // Current block height
        let blocks_per_day = 144;  // Estimate of blocks per day on Bitcoin network

        // Iterate over the last 7 days
        for day in 0..1 {
            // Calculate the block range for the day
            let start_block = current_block - (day + 1) * blocks_per_day;
            let end_block = current_block - day * blocks_per_day;

            let mut daily_tx_count = 0;
            let mut block_date = None;

            // Iterate over the blocks in the day
            for block_height in start_block..=end_block {
                let block_hash = match self.rpc_client.get_block_hash(block_height) {
                    Ok(hash) => hash,
                    Err(e) => {
                        eprintln!("Error retrieving block hash at height {}: {:?}", block_height, e);
                        continue; // Skip this block and continue with others
                    }
                };
                let block = match self.rpc_client.get_block(&block_hash) {
                    Ok(block) => block,
                    Err(e) => {
                        eprintln!("Error retrieving block data for hash {}: {:?}", block_hash, e);
                        continue; // Skip this block and continue with others
                    }
                };

                // Only retrieve the block date once for the first block
                let block_header = match self.rpc_client.get_block_header(&block_hash) {
                    Ok(header) => header,
                    Err(e) => {
                        eprintln!("Error retrieving block header for hash {}: {:?}", block_hash, e);
                        continue; // Skip this block and continue with others
                    }
                };

                if block_date.is_none() {
                    let block_time = block_header.time;  // Block time in Unix timestamp
                    let date = NaiveDateTime::from_timestamp(block_time as i64, 0).date();
                    block_date = Some(date);
                }

                // Increment the transaction count for the current block
                daily_tx_count += block.txdata.len();
            }

            // Push the result if the date was successfully set
            if let Some(date) = block_date {
                daily_tx_counts.push(DailyTxData {
                    date,
                    tx_count: daily_tx_count,
                });
            }
        }

        Ok(daily_tx_counts)
    }

    pub async fn get_fee_estimation(&self, block_target: u16) -> Result<f64, Box<dyn std::error::Error>> {
        let fee_estimate = self.rpc_client.estimate_smart_fee(block_target, Some(EstimateMode::Conservative))?;
        if let Some(fee_rate) = fee_estimate.fee_rate {
            Ok(fee_rate.to_sat() as f64 / 1000.0)  // Convert to satoshis per byte if necessary
        } else {
            Err("Fee rate not available".into())
        }
    }

}


