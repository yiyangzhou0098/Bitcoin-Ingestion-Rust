use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::sync::Arc;

pub struct BitcoinRpcService {
    rpc_client: Client,
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
}
