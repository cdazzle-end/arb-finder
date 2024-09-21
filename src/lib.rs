
use asset_registry_2::MyAsset;
use bigdecimal::BigDecimal;
// mod asset_registry;
// mod liq_pool_registry;
// mod adjacency_table;
// mod token_graph;
// mod token;
// use token::{Token, AssetKeyType};
// use adjacency_table::{AdjacencyTable};
// use asset_registry::AssetRegistry;
// use liq_pool_registry::LiqPoolRegistry;
// use token_graph::TokenGraph;
// use token_graph::calculate_swap;
use futures::future::join_all;
// mod constants;
mod fee_book;
mod asset_registry_2;
mod liq_pool_registry_2;
mod adjacency_table_2;
mod token_graph_2;
mod result_logger;
pub mod utils;
pub mod constants;
use adjacency_table_2::{AdjacencyTable2};
use asset_registry_2::AssetRegistry2;
use liq_pool_registry_2::LiqPoolRegistry2;
use num::BigInt;
use num::FromPrimitive;
use token_graph_2::get_sqrt_ratio_at_tick;
use token_graph_2::ArbBestPath;
use token_graph_2::PathData;
use token_graph_2::TokenGraph2;
use token_graph_2::GraphNode;
use result_logger::ResultLogger;
pub use utils::Relay;
pub use constants::*;

use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use std::str;
use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::{join, task};
use std::fs::OpenOptions;
use log::error;

use crate::asset_registry_2::Asset;
use crate::token_graph_2::PathNode;
// use std::io::prelude::*;
type NodePath = Vec<Rc<RefCell<GraphNode>>>;

#[derive(Debug, Clone)]
pub struct IsolatedSearchResult {
    input_amount: BigDecimal,
    display_string: String,
    path_nodes: Vec<PathNode>
}


/// Run target search from relay node with 3 different inputs 
pub async fn run_async_default_searches(relay: Relay){
    let asset_key = match relay {
        Relay::Polkadot => constants::POLKADOT_START_NODE.to_string(),
        Relay::Kusama => constants::KUSAMA_START_NODE.to_string(),
    };
    let start_asset = utils::get_asset_by_asset_key(asset_key.clone(), relay);

    let inputs = vec![
        ("small", BigDecimal::from_f64(0.5).unwrap()),
        ("medium", BigDecimal::from_f64(2.0).unwrap()),
        ("big", BigDecimal::from_f64(5.0).unwrap()),
    ];
    let xcm_start_nodes = true;

    // Call target search with different inputs, return size and search result
    let handles: Vec<_> = inputs
        .into_iter()
        .map(|(size, input)| {
            let start_key = asset_key.clone();
            let destination_key = asset_key.clone();
            let relay = relay.clone();

            task::spawn(async move {
                let result = target_search(start_key, destination_key, input, relay, xcm_start_nodes).await;
                (size, result)
            })
        })
        .collect();

    let results = join_all(handles).await;

    for result in results {
        match result {
            Ok((size, search_result)) => {
                println!("Result for {} input:", size);
                println!("Input amount: {}", search_result.input_amount);
                println!("Final path value: {}", search_result.path_nodes.last().unwrap().path_value);
                println!("---");
                ResultLogger::log_results_default(
                    search_result.path_nodes, 
                    start_asset.get_asset_name().clone().to_string(), 
                    search_result.input_amount, 
                    relay
                );
    
            }
            Err(e) => eprintln!("Task failed with error: {:?}", e),
        }
    }
}

/// Run target arb from start key to destination key
/// - Main entry point for arb-executor to find a new arb
/// 
/// Log results in target folder
pub async fn run_and_log_target_search(start_key: String, destination_key: String, input_amount: BigDecimal, relay: Relay){
    let xcm_start_nodes = true;
    let search_result: IsolatedSearchResult = target_search(start_key.clone(), destination_key, input_amount, relay, xcm_start_nodes).await;

    let start_asset = utils::get_asset_by_asset_key(start_key.clone(), relay);

    ResultLogger::log_results_target(
        search_result.path_nodes.clone(),
        start_asset.get_asset_name().to_string(),
        relay
    );
}
/// Run target arb from start key to destination key
/// 
/// Log results in fallback folder
pub async fn run_and_log_fallback_search(relay: Relay, start_key: String, destination_key: String, input_amount: BigDecimal){
    let xcm_start_nodes = false;
    let search_result: IsolatedSearchResult = target_search(start_key.clone(), destination_key.clone(), input_amount, relay, xcm_start_nodes).await;

    let destination_asset = utils::get_asset_by_asset_key(destination_key.clone(), relay);

    let loggable_results = ResultLogger::log_results_fallback(
        search_result.path_nodes, 
        destination_asset.get_asset_name().to_string(), 
        relay
    );
    println!("{}", search_result.display_string);
}

/// Target search. Find best path from start node to destination node with specified input
/// - Get's all start nodes from start_key
/// - Calls isolated_search for each start node with specified input
/// - return IsolatedSearchResult for highest value path
/// 
/// execute_with_xcm_start_nodes to use all start nodes, or just designated start node
pub async fn target_search(start_key: String, destination_key: String, input_amount: BigDecimal, relay: Relay, execute_with_xcm_start_nodes: bool) -> IsolatedSearchResult {
    let start_asset = utils::get_asset_by_asset_key(start_key.clone(), relay);
    let mut xcm_assets: Vec<MyAsset> = utils::get_assets_at_location(start_asset.clone(), relay);

    // SKIP GLMR TEST
    // xcm_assets.retain(|asset| asset.get_chain_id() != Some(2004));

    let start_nodes = if execute_with_xcm_start_nodes {
        xcm_assets.clone()
    } else {
        vec![start_asset.clone()]
    };

    // Run search from each start node, collect each task
    let handles = start_nodes.into_iter().map(|asset| {
        let node_key = utils::get_asset_key(asset);
        let relay_clone = relay.clone();
        let dest_key = destination_key.clone();
        let amount = input_amount.clone();

        println!("Executing isolated search for {} | {}", node_key, amount);

        task::spawn(async move {
            isolated_search(relay_clone, node_key, dest_key, amount).await
        })
    }).collect::<Vec<_>>();

    // Await all results
    let results = join_all(handles).await;

    // Extract IsolatedSearchResult values from future results 
    let search_results: Vec<IsolatedSearchResult> = results.into_iter()
        .filter_map(|result| match result {
            Ok(ok) => Some(ok),
            Err(e) => {
                panic!("Task failed with error: {:?}", e);
                None
            },
        })
    .collect();

    // Get the result with the highest path value
    let highest_search_result = search_results.iter()
        .max_by_key(|result| BigDecimal::from_str(&result.path_nodes.last().unwrap().path_value).unwrap())
        .expect("No valid search results found");

    // Print all paths
    for result in &search_results {
        println!("*****************************************");
        println!("Input value: {} | Final path value: {}", result.input_amount, result.path_nodes.last().unwrap().path_value);
        for node in &result.path_nodes {
            println!("{}: {} {}", node.node_key, node.asset_name, node.path_value);
        }
        println!("*****************************************");
    }

    // Print highest value
    println!("Highest path: {}", highest_search_result.path_nodes.last().unwrap().path_value);
    for node in &highest_search_result.path_nodes {
        println!("{}: {} {}", node.node_key, node.asset_name, node.path_value);
    }
    println!("---- End highest path");

    highest_search_result.clone()
}

/// Runs synchronous searches, one by one, from each relay token start node with specified input amount (1)
/// - Useful for testing purposes
pub async fn search_default_sync(relay: Relay){
    let relay = Relay::Polkadot;
    let start_key = "2030{\"Token2\":\"0\"}".to_string();
    let destination_key = "2000{\"NativeAssetId\":{\"Token\":\"DOT\"}}".to_string();

    let graph = utils::build_token_graph(relay);

    let start_node = &graph.get_node(start_key.clone()).clone();
    
    let start_node_asset_name = start_node.borrow().get_asset_name();

    let start_asset_location = start_node.borrow().get_asset_location().unwrap();
    let all_start_assets = &graph.asset_registry.get_assets_at_location(start_asset_location);

    // search_best_path_a_to_b_sync_polkadot(start_key.clone(), start_key, 1 as f64);

    //***************************************** */
    let mut start_nodes = vec![];
    // let mut inputAmounts = vec![];
    for start_asset in all_start_assets{
        if !start_asset.borrow().is_cex_token() {
            let new_start_node = &graph.get_node(start_asset.borrow().get_map_key()).clone();
            start_nodes.push(Rc::clone(&new_start_node));
        }
    }

    let input_amount = BigDecimal::from(1);

    for node in start_nodes.clone(){
        let key = node.borrow().get_asset_key();
        println!("Searching for {}", key);
        let dest_key = destination_key.clone();
        let amount = input_amount.clone();
        // let (value, display, path) = search_best_path_a_to_b_sync_polkadot(key, dest_key, amount);
        let search_result: IsolatedSearchResult = isolated_search(relay, key, dest_key, amount).await;
    }
    // *********************************************


}



/// Main search function that can take either relay
/// 
/// Build new token graph for specified relay
/// 
/// Runs one search for specified start key 
pub async fn isolated_search(relay: Relay, start_key: String, destination_key: String, input_amount: BigDecimal) -> IsolatedSearchResult {
    let graph: TokenGraph2 = utils::build_token_graph(relay);
    let arb_result: ArbBestPath = graph.find_best_route(start_key, destination_key, input_amount.clone());

    let return_path: Vec<PathNode> = utils::return_path_nodes(arb_result.best_path);

    let search_results = IsolatedSearchResult {
        input_amount: input_amount.clone(),
        display_string: arb_result.display_string,
        path_nodes: return_path
    };
    search_results
}

/// Run's fallback search and saves log in fallback log folder
/// - Need to rework nameing convention, previously it was taking the last node in the path and using the asset name of that for the file name
/// - Now just using the name of the destination asset name
/// - Need to rewrite this, if any of it is necessary
pub async fn fallback_search( relay: Relay, start_key: String, destination_key: String, input_amount: BigDecimal){
    let destination_asset = utils::get_asset_by_asset_key(start_key.clone(), Relay::Polkadot);

    let graph = utils::build_token_graph(relay);
    
    let arb_result: ArbBestPath = graph.find_best_route(start_key, destination_key, input_amount);

    let return_path = utils::return_path_nodes(arb_result.best_path);
    

    let loggable_results = ResultLogger::log_results_fallback(return_path, destination_asset.get_asset_name().to_string(), relay);
    println!("{}", arb_result.display_string);


}

pub fn test_builder(relay: Relay){

    let graph = utils::build_token_graph(relay);

    // graph.asset_registry.display_all_assets();
}

pub fn single_search(relay: Relay){

    let asset_key;

    match relay {
        Relay::Polkadot => asset_key = constants::POLKADOT_START_NODE.to_string(),
        Relay::Kusama => asset_key = constants::KUSAMA_START_NODE.to_string(),
    }
    
    let input_amount = BigDecimal::from_f64(0.5 as f64).unwrap();

    let graph = utils::build_token_graph(relay);
    let arb_result: ArbBestPath = graph.find_best_route(asset_key.clone(), asset_key.clone(), input_amount.clone());

    let arb_finder_path = utils::return_path_nodes(arb_result.best_path);

    for node in arb_finder_path {
        println!("{:#?}", node);
    }
}





pub async fn print_asset_keys(start_key: String, relay: Relay){
    let mut asset_registry = AssetRegistry2::build_asset_registry();    
    let graph = utils::build_token_graph(relay);
    graph.get_asset_keys(start_key);
}

pub fn test_utils(){
    let start_key = "2000{\"NativeAssetId\":{\"Token\":\"DOT\"}}".to_string();
    let test_asset = utils::get_asset_by_asset_key(start_key, Relay::Polkadot);

    let assets_at_location = utils::get_assets_at_location(test_asset, Relay::Polkadot);

    println!("Asset: {:#?}", assets_at_location);
}
