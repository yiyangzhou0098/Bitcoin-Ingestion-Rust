// ingestion.rs

use std::sync::Arc;
use chrono::{Utc, NaiveDate};
use crate::{MySqlService, BitcoinRpcService}; // Import your services from main.rs

// Function to calculate 7DMA from the last 7 days' transaction data
fn calculate_7dma(transaction_data: &[(NaiveDate, usize)]) -> f64 {
    let sum: usize = transaction_data.iter().map(|(_, tx_count)| *tx_count).sum();
    sum as f64 / transaction_data.len() as f64 // Average the transactions over the 7 days
}

// Function to retrieve the latest data and store it in MySQL before starting the server
pub async fn retrieve_and_store_data(
    mysql_service: Arc<MySqlService>,
    bitcoin_service: Arc<BitcoinRpcService>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Retrieving latest data...");

    // Retrieve the last 7 days' transaction data from Bitcoin RPC
    // let transaction_data = bitcoin_service.get_last_7_days_tx_data().await?;
    // // Save the transaction data in MySQL
    // for data in transaction_data {
    //     mysql_service.save_daily_tx(data.date, data.tx_count).await?;
    // }

    // Optionally, calculate and store 7DMA if needed
    // let last_7_days_data: Vec<(NaiveDate, usize)> = mysql_service.get_last_7_days().await?;
    // let dma_value = calculate_7dma(&last_7_days_data);
    // let today = Utc::now().naive_utc().date();
    // mysql_service.save_7dma(today, dma_value).await?;
    
    retrieve_and_store_fee_estimations(mysql_service.clone(), bitcoin_service.clone()).await?;


    println!("Data retrieval and storage completed.");
    Ok(())
}

 // Function to retrieve fee estimation data and store it in MySQL
 pub async fn retrieve_and_store_fee_estimations(
    mysql_service: Arc<MySqlService>,
    bitcoin_service: Arc<BitcoinRpcService>,
) -> Result<(), Box<dyn std::error::Error>> {
    let block_targets = vec![1, 3, 6, 12, 24]; // Different block targets for fee estimation

    for block_target in block_targets {
        match bitcoin_service.get_fee_estimation(block_target).await {
            Ok(fee_rate) => {
                println!("Fee rate for block target {}: {} sat/byte", block_target, fee_rate);
                mysql_service.save_fee_estimation(block_target, fee_rate).await?;
            }
            Err(e) => eprintln!("Error retrieving fee estimation for block target {}: {}", block_target, e),
        }
    }

    Ok(())
}
