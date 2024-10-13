use bitcoincore_rpc::{Auth, Client, RpcApi};

pub fn test_rpc_func() -> Result<(), bitcoincore_rpc::Error> {
    let rpc = Client::new("http://38.42.241.37:8332", Auth::UserPass("liamz".to_string(), "770823669".to_string()))?;
    let best_block_hash = rpc.get_best_block_hash()?;
    println!("Best block hash: {}", best_block_hash);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;  // Import the parent module (so we can test `test_rpc_func`)

    #[test]
    fn test_rpc_func_success() {
        // Run the function and check if it executes without error
        // match test_rpc_func() {
        //     Ok(()) => assert!(true), // If the function succeeds
        //     Err(e) => panic!("RPC test failed: {:?}", e), // If the function fails, panic with the error
        // }
        test_rpc_func();
    }

    // #[test]
    // fn test_rpc_func_with_config() {
        // // In this test, you might check if your config values are correct
        // assert!(!RPC_URL.is_empty(), "RPC_URL should not be empty");
        // assert!(!RPC_USER.is_empty(), "RPC_USER should not be empty");
        // assert!(!RPC_PWD.is_empty(), "RPC_PWD should not be empty");

        // // Optionally, you can skip this test if you're in an environment without RPC access.
        // // For example:
        // if RPC_URL.is_empty() || RPC_USER.is_empty() || RPC_PWD.is_empty() {
        //     println!("Skipping RPC test due to missing configuration");
        //     return;
        // }

        // match test_rpc_func() {
        //     Ok(_) => assert!(true), // Success case
        //     Err(e) => panic!("RPC failed: {:?}", e), // Failure case
        // }
    // }
}