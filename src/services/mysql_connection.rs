use mysql::*;
use mysql::prelude::*;
use std::{string, sync::Arc};
use chrono::{NaiveDate, Utc, Datelike};

pub struct MySqlService {
    pool: Pool,
}

impl MySqlService {
    pub fn new(database_url: &str) -> Arc<Self> {
        let pool = Pool::new(database_url).expect("Failed to create MySQL pool");
        Arc::new(Self { pool })
    }

    pub fn update_block_height(&self, block_height: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let query = r"INSERT INTO block_info (block_height) VALUES (?) 
                      ON DUPLICATE KEY UPDATE block_height = VALUES(block_height)";
        conn.exec_drop(query, (block_height,))?;
        Ok(())
    }

    // Save the daily transaction data (with date) to MySQL
    pub async fn save_daily_tx(&self, date: NaiveDate, tx_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;

        conn.exec_drop(
            r"INSERT INTO daily_transactions (date, tx_count)
              VALUES (:date, :tx_count)
              ON DUPLICATE KEY UPDATE tx_count = :tx_count",
            params! {
                "date" => date.format("%Y-%m-%d").to_string(),
                "tx_count" => tx_count,
            },
        )?;

        Ok(())
    }

    // Fetch the last 7 days of transaction data from MySQL
    pub async fn get_last_7_days(&self) -> Result<Vec<(NaiveDate, usize)>, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        // Fetch the data from MySQL as (String, usize) and then parse the date string into NaiveDate
        let result = conn.query_map(
            "SELECT date, tx_count FROM daily_transactions ORDER BY date DESC LIMIT 7",
            |(date_str, tx_count): (String, usize)| {
                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
                (date, tx_count)
            },
        )?;
        Ok(result)
    }

    // Fetch the last 7 days of transaction data from MySQL
    pub async fn get_all_days_tx(&self) -> Result<Vec<(NaiveDate, usize)>, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        // Fetch the data from MySQL as (String, usize) and then parse the date string into NaiveDate
        let result = conn.query_map(
            "SELECT date, tx_count FROM daily_transactions_test ORDER BY date DESC",
            |(date_str, tx_count): (String, usize)| {
                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
                (date, tx_count)
            },
        )?;
        Ok(result)
    }

    // Check if today's transaction data exists in MySQL
    pub fn check_today_data(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let today = Utc::now().naive_utc().date(); // Get today's date
        let formatted_date = format!("{}-{:02}-{:02}", today.year(), today.month(), today.day());
        

        let result: Option<(String,)> = conn.exec_first(
            "SELECT date FROM daily_transactions WHERE date = :date",
            params! {
                "date" => formatted_date,
            },
        )?;

        Ok(result.is_some()) // Return true if data exists, false otherwise
    }

        // Save today's transaction data (update the latest block height data)
    pub fn save_today_tx(&self, tx_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let today = Utc::now().naive_utc().date(); // Get today's date

        conn.exec_drop(
            r"INSERT INTO daily_transactions (date, tx_count)
                VALUES (:date, :tx_count)
                ON DUPLICATE KEY UPDATE tx_count = :tx_count",
            params! {
                "date" => today.format("%Y-%m-%d").to_string(),
                "tx_count" => tx_count,
            },
        )?;

        Ok(())
    }

        // Save the calculated 7DMA data for a specific date
    pub async fn save_7dma(&self, date: NaiveDate, dma_value: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(
            r"INSERT INTO seven_day_dma (date, dma_value)
                VALUES (:date, :dma_value)
                ON DUPLICATE KEY UPDATE dma_value = :dma_value",
            params! {
                "date" => date.format("%Y-%m-%d").to_string(),
                "dma_value" => dma_value,
            },
        )?;
        Ok(())
    }

    /* -------------------- Off chain data operations -------------------- */
    // Save fee estimation data into MySQL
    pub async fn save_fee_estimation(&self, block_target: u16, fee_rate: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get_conn()?;
        let estimated_at = Utc::now().naive_utc().format("%Y-%m-%d").to_string();
        
        conn.exec_drop(
            r"INSERT INTO fee_estimations (block_target, fee_rate, estimated_at) 
                VALUES (:block_target, :fee_rate, :estimated_at)
                ON DUPLICATE KEY UPDATE fee_rate = :fee_rate, estimated_at = :estimated_at",
            params! {
                "block_target" => block_target,
                "fee_rate" => fee_rate,
                "estimated_at" => estimated_at,
            },
        )?;
        
        Ok(())
    }

    // Fetch all fee estimation data from MySQL
    pub async fn get_fee_estimations(&self) -> Result<Vec<(u16, f64, String)>, Box<dyn std::error::Error>> {
        // Get a connection from the pool
        let mut conn = self.pool.get_conn()?;
    
        // Execute the query asynchronously and await the result
        let fee_estimations = conn.query_map(
            "SELECT block_target, fee_rate, estimated_at FROM fee_estimations",
            |(block_target, fee_rate, estimated_at): (u16, f64, String)| {
                (block_target, fee_rate, estimated_at)
            },
        )?;
    
        // Return the result
        Ok(fee_estimations)
    }


}
