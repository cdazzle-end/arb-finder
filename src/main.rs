// use core::num::dec2flt::parse;
// mod token;

use arb_handler::*;
use std::collections::HashMap;
// use asset_registry::AssetRegistry;
use std::fs::File;
use std::io::prelude::*;
use serde_json::{Value};
use std::{str, process};
use std::path::Path;
use num::{BigInt, BigUint, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, FromPrimitive, Num, One, Signed, ToPrimitive, Zero};
use bigdecimal::{BigDecimal};
use std::str::FromStr;

use utils::Relay;
// use constants::*;
// mod utils;
// use utils::Relay;
// use tokio::{join, task};
// mod liq_pool;

// use liq_pool::LiqPool;
// cargo run search_best_path_a_to_b "2001{\`"Native\`":\`"BNC\`"}" "2000{\`"NativeAssetId\`":{\`"Token\`":\`"KSM\`"}}" 10
// cargo run search_best_path_a_to_b "2001{\`"Native\`":\`"BNC\`"}" "2000{\`"NativeAssetId\`":{\`"Token\`":\`"KSM\`"}}" 10
// cargo run search_best_path_a_to_b_kusama "2110\`"26\`"" "2000{\`"NativeAssetId\`":{\`"Token\`":\`"KSM\`"}}" 1
// cargo run search_best_path_a_to_b_kusama "2110\`"26\`"" "2110\`"26\`"" 1400
// cargo run search_best_path_a_to_b_kusama "2110\`"4\`"" "2110\`"26\`"" 0.5
// cargo run search_best_path_a_to_b_kusama "2001{\`"Token\`":\`"ZLK\`"}" "2110\`"26\`"" 700
// cargo run search_best_path_a_to_b_kusama "2001{\`"Token\`":\`"ZLK\`"}" "2110\`"4\`"" 700
// cargo run search_best_path_a_to_b_polkadot "2000{\`"NativeAssetId\`":{\`"Token\`":\`"DOT\`"}}" "2000{\`"NativeAssetId\`":{\`"Token\`":\`"DOT\`"}}" 1
// cargo run fallback_search_a_to_b_polkadot "2034\`"102\`"" "2000{\`"NativeAssetId\`":{\`"Token\`":\`"DOT\`"}}" 2.404927102023512903
//     let key_1 = "2000{\"ForeignAssetId\":\"0\"}".to_string();
//     let key_1 = "2023\"MOVR\"".to_string();
//     let key_1 = "2000{\"NativeAssetId\":{\"Token\":\"KSM\"}}".to_string();
// cargo run target_search polkadot "2000{`\`"NativeAssetId`\`":{`\`"Token\`":`\`"DOT`\`"}}" "2000{`\`"NativeAssetId`\`":{`\`"Token\`":`\`"DOT`\`"}}"  1 
// cargo run fallback_search polkadot "2000{`\`"NativeAssetId`\`":{`\`"Token\`":`\`"DOT`\`"}}" "2000{`\`"NativeAssetId`\`":{`\`"Token\`":`\`"DOT`\`"}}"  1


#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            // Main function
            // Runs a search from specified start node to destination node for the input amount.
            // Input: Relay, start key, dest key, input amount
            "target_search" if args.len() == 6 => {
                let relay: Relay = Relay::from_str(&args[2]);
                let start_key = &args[3];
                let destination_key = &args[4];
                let input_amount_str = &args[5];
                println!("Relay {:?} | start: {} | destination: {} | input amount: {}",relay, start_key, destination_key, input_amount_str);
                let input_amount_bd = BigDecimal::from_str(input_amount_str)
                    .expect("Input amount must be a valid number");
                run_and_log_target_search(start_key.clone(), destination_key.clone(), input_amount_bd, relay).await;
            },
            /// Same as target search, but saves logs results in fallback log folder
            "fallback_search" if args.len() == 6 => {
                let relay: Relay = Relay::from_str(&args[2]);
                let start_key = &args[3];
                let destination_key = &args[4];
                let input_amount_str = &args[5];
                println!("Relay {:?} | start: {} | destination: {} | input amount: {}",relay, start_key, destination_key, input_amount_str);
                
                let input_amount_bd = BigDecimal::from_str(input_amount_str)
                    .expect("Input amount must be a valid number");
                run_and_log_fallback_search(relay, start_key.to_string(), destination_key.to_string(), input_amount_bd).await;
            },
            // Input: Relay, Input Amount. --- Run a fresh arb from relay start node to end node, with specified input or default amount if ommitted
            "default_search" => {
                let relay: Relay = Relay::from_str(&args[2]);
                let asset_key = match relay {
                    Relay::Polkadot => constants::POLKADOT_START_NODE,
                    Relay::Kusama => constants::KUSAMA_START_NODE,
                };
                let input_amount_bd = if args.len() > 3 {
                    BigDecimal::from_str(&args[3])
                        .expect("Input amount must be a valid number")
                } else {
                    BigDecimal::from(1) // Default value if no amount is provided
                };
                let xcm_start_nodes = true;
                target_search(asset_key.to_string(), asset_key.to_string(), input_amount_bd, relay, xcm_start_nodes).await;
            },
            
            /// Run 3 searches from relay node to relay node with 3 different input values
            "async_default_search" => {
                let relay: Relay = Relay::from_str(&args[2]);
                run_async_default_searches(relay).await;
            },
            "search_polkadot_sync" => {
                println!("Running polkadot search SYNC. One by one");
                let asset_key = "2000{\"NativeAssetId\":{\"Token\":\"DOT\"}}".to_string();
                search_default_sync(Relay::Polkadot).await;
            },
            "p_1" => {
                let asset_key = "2000{\"NativeAssetId\":{\"Token\":\"DOT\"}}".to_string();
                // search_best_path_a_to_b_polkadot(asset_key.clone(), asset_key, BigDecimal::from(1)).await;
            },
            "test" => {
                let relay = Relay::Polkadot;
                // default_target_search().await;
                run_async_default_searches(relay).await;
                // search_best_path_a_to_b_sync_polkadot().await
                // single_search(relay);
                // test_builder(relay)
            },
            _ => {
                eprintln!("Error: search_best_path_a_to_b incorrect parameters"); // Write an error message to stderr
                process::exit(1); // Exit with a non-zero status code to indicate failure
            }
        }
    } else {
        println!("No arguments provided. Running default function.");
        // async_search_default_kusama().await;
    }

}
// #[tokio::main]
// async fn main(){
//     let asset_key = "2000{\"NativeAssetId\":{\"Token\":\"KSM\"}}".to_string();
//     let polkadot_assets = test_polkadot_assets();
// }


fn clean_string(s: &str) -> &str{
    //remove brackets
    &s[1..s.len()-1]
}

//Read json from kar_asset_registry file




