use mysql::*;
use mysql::prelude::*;
use std::sync::Arc;

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
}
