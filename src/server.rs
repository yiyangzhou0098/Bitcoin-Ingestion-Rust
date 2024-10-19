use warp::Filter;
use std::sync::Arc;
use crate::services::mysql_connection::MySqlService;
use crate::services::bitcoin_rpc::BitcoinRpcService;
use serde::Serialize;
use chrono::NaiveDate;

use warp::reject::Reject;
use std::fmt;


//////////////////////////////////////////
// Define a custom error type
#[derive(Debug)]
struct CustomError {
    message: String,
}
//////////////////////////////////////////
#[derive(Serialize)]
struct TxData {
    date: NaiveDate,
    tx_count: usize,
}

#[derive(Serialize)]
struct FeeRateData {
    block_target: u16,
    fee_rate: f64,
    estimated_at: String,
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

    let tx_data_route = warp::path!("api" / "7d_tx")
        .and(warp::get())
        .and(with_mysql_service(mysql_service.clone()))
        .and_then(handle_get_last_7_days);

    let fee_estimations_route = warp::path!("api" / "fee_estimations")
        .and(warp::get())
        .and(with_mysql_service(mysql_service.clone()))
        .and_then(handle_get_fee_estimations);

    let routes = 
                get_block_height_route
                .or(tx_data_route)
                .or(fee_estimations_route);

    // Start the warp server
    warp::serve(routes)
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

fn with_mysql_service(
    mysql_service: Arc<MySqlService>,
) -> impl Filter<Extract = (Arc<MySqlService>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || mysql_service.clone())
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

// Route handler for getting the last 7 days' data
async fn handle_get_last_7_days(
    mysql_service: Arc<MySqlService>, // We pass MySqlService via Arc
) -> Result<impl warp::Reply, warp::Rejection> {
    // Step 1: Fetch the last 7 days of transaction data from MySQL
    let last_7_days_data = match mysql_service.get_all_days_tx().await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to fetch data: {}", err);
            let custom_error = CustomError {
                message: format!("Failed to fetch data: {:?}", err),
            };
            return Err(warp::reject::custom(custom_error)); // Handle database errors gracefully
        }
    };

    // Step 2: Transform the data into a response-friendly format
    let response_data: Vec<TxData> = last_7_days_data.into_iter()
        .map(|(date, tx_count)| TxData { date, tx_count })
        .collect();

    // Step 3: Return the data as a JSON response
    Ok(warp::reply::json(&response_data))
}

// Route handler to return fee estimation data as JSON
pub async fn handle_get_fee_estimations(
    mysql_service: Arc<MySqlService>
) -> Result<impl warp::Reply, warp::Rejection> {
    match mysql_service.get_fee_estimations().await {
        Ok(fee_estimations) => {
            let response_data: Vec<FeeRateData> = fee_estimations
                .into_iter()
                .map(|(block_target, fee_rate, estimated_at)| FeeRateData {
                    block_target,
                    fee_rate,
                    estimated_at,
                })
                .collect();

            Ok(warp::reply::json(&response_data))
        }
        Err(e) => {
            eprintln!("Failed to fetch data: {}", e);
            let custom_error = CustomError {
                message: format!("Failed to fetch data: {:?}", e),
            };
            return Err(warp::reject::custom(custom_error)); // Handle database errors gracefully
        } 
    }
}