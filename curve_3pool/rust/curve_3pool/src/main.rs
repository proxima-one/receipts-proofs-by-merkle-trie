use dotenv::dotenv;
use std::env;

extern crate web3;

use web3::types::{Block, BlockId, Transaction, H256};

#[tokio::main]
async fn main() {
    // Loading environment variables from .env file
    dotenv().ok();

    let infura_api_key = match env::var("INFURA_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            println!("Failed to get INFURA_API_KEY value.");
            return;
        }
    };

    let infura_url = format!("https://mainnet.infura.io/v3/{}", infura_api_key);
//     println!("infura_url: {}", infura_url);

    let transport = web3::transports::Http::new(&infura_url).unwrap();
    let web3 = web3::Web3::new(transport);

    let last_block_number = match web3.eth().block_number().await {
        Ok(val) => val,
        Err(err) => {
            println!("Failed to get last block number: {}", err);
            return;
        }
    };

    println!("Last block number: {:?}", last_block_number);

    let block_id = BlockId::Number(last_block_number.into());

    let block: Option<Block<Transaction>> = match web3.eth().block_with_txs(block_id).await {
        Ok(block) => block,
        Err(err) => {
            println!("Failed to get block: {}", err);
            return;
        }
    };

    if let Some(block) = block {
        let block_hash = block.hash;
        let parent_hash = block.parent_hash;
        let receipts_root = block.receipts_root;
        let transactions_root = block.transactions_root;
        let state_root = block.state_root;
        let timestamp = block.timestamp;
        let gas_used = block.gas_used;

        println!("Block Hash: {:?}", parent_hash);
        println!("Parent Block Hash: {:?}", block_hash);
        println!("Receipts Root: {:?}", receipts_root);
        println!("Transactions Root: {:?}", transactions_root);
        println!("State Root: {:?}", state_root);
        println!("Timestamp: {:?}", timestamp);
        println!("Gas Used: {:?}", gas_used);
    } else {
        println!("Block number {} not found.", last_block_number);
    }

    // https://etherscan.io/tx/0xd82caa2189d8987db426569ab12a261d93872ec472c03dac39515e3a42a4e668#eventlog
    let transaction_hash = H256::from_slice(
        &hex::decode("d82caa2189d8987db426569ab12a261d93872ec472c03dac39515e3a42a4e668")
            .expect("Failed to decode transaction hash"),
    );

//     let receipt = match web3.eth().transaction_receipt(transaction_hash).await {
//         Ok(val) => val,
//         Err(err) => {
//             println!("Failed to get transaction receipt: {}", err);
//             return;
//         }
//     };
//     println!("Transaction receipt: {:?}", receipt);

    let receipt_option = web3.eth().transaction_receipt(transaction_hash).await.unwrap();

    if let Some(receipt) = receipt_option {
        let address = "0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7".to_lowercase();

        let filtered_logs: Vec<_> = receipt.logs.iter()
            .filter(|log| format!("{:?}", log.address).to_lowercase() == address)
            .collect();

        for log in filtered_logs {
            println!("{:?}", log);
        }
    }
}
