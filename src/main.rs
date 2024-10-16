mod services;
mod server;
mod config;

use config::connections::{RPC_URL,RPC_PWD,RPC_USER,DB_URL};
use services::mysql_connection::MySqlService;
use services::bitcoin_rpc::BitcoinRpcService;
use services::ingestion::retrieve_and_store_data;
use server::run_server;
use tokio::main;

#[main]
async fn main() {
    // Set up MySQL connection
    let mysql_service = MySqlService::new(DB_URL);

    // Set up Bitcoin RPC connection
    let bitcoin_service = BitcoinRpcService::new(
        RPC_URL,
        RPC_USER,
        RPC_PWD
    );

    // Step 1: Retrieve the latest data before starting the server
    if let Err(e) = retrieve_and_store_data(mysql_service.clone(), bitcoin_service.clone()).await {
        eprintln!("Error retrieving and storing initial data: {:?}", e);
        return;
    }

    // Step 2: Run the Warp server
    run_server(mysql_service, bitcoin_service).await;
}

// extern crate bitcoincore_rpc;
// use bitcoincore_rpc::{Auth, Client, RpcApi};

// mod config;
// use config::connections::{RPC_URL,RPC_PWD,RPC_USER};
// fn main() {

//     let rpc = Client::new(RPC_URL,
//                           Auth::UserPass(RPC_USER.to_string(),
//                                          RPC_PWD.to_string())).unwrap();
//     let best_block_hash = rpc.get_best_block_hash().unwrap();
//     println!("best block hash: {}", best_block_hash);
// }