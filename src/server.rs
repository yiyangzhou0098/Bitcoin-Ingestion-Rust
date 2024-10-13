use warp::Filter;
use std::sync::Arc;
use crate::services::mysql_connection::MySqlService;
use crate::services::bitcoin_rpc::BitcoinRpcService;
use serde::Serialize;

use warp::reject::Reject;
use std::fmt;


//////////////////////////////////////////
// Define a custom error type
#[derive(Debug)]
struct CustomError {
    message: String,
}

// Implement Reject for CustomError to use it in warp::reject::custom
impl Reject for CustomError {}

// Implement Display and Error traits for better error reporting
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CustomError {}

//////////////////////////////////////////

#[derive(Serialize)]
struct BlockHeightResponse {
    block_height: u64,
}

// Function to create the Warp REST API server
pub async fn run_server(mysql_service: Arc<MySqlService>, bitcoin_service: Arc<BitcoinRpcService>) {
    // Define a route to fetch the latest block height
    let get_block_height_route = warp::path!("api" / "block_info" / "block_height")
        .and(warp::get())
        .and(with_services(mysql_service.clone(), bitcoin_service.clone()))
        .and_then(handle_get_block_height);

    // Start the warp server
    warp::serve(get_block_height_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

// Helper function to inject services into the route handler
fn with_services(
    mysql_service: Arc<MySqlService>,
    bitcoin_service: Arc<BitcoinRpcService>
) -> impl Filter<Extract = ((Arc<MySqlService>, Arc<BitcoinRpcService>),), Error = warp::Rejection> + Clone {
    warp::any()
        .map(move || (mysql_service.clone(), bitcoin_service.clone()))
        .boxed()
}

// Route handler to fetch and return the block height
async fn handle_get_block_height(
    services: (Arc<MySqlService>, Arc<BitcoinRpcService>)
) -> Result<impl warp::Reply, warp::Rejection> {
    let (mysql_service, bitcoin_service) = services;

    match bitcoin_service.get_block_height() {
        Ok(block_height) => {
            // Update the block height in MySQL
            if let Err(e) = mysql_service.update_block_height(block_height) {
                eprintln!("Failed to update block height in MySQL: {:?}", e);
            }

            // Return the block height as JSON response
            let response = BlockHeightResponse { block_height };
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            eprintln!("Failed to retrieve block height: {:?}", e);
            let custom_error = CustomError {
                message: format!("Bitcoin RPC error: {:?}", e),
            };
            Err(warp::reject::custom(custom_error))
        }
    }
}
