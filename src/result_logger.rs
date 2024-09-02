use crate::token_graph_2::{PathData, PathType};
use crate::utils::Relay;
use crate::{NodePath, PathNode};
use std::fs::File;
use std::io::prelude::*;
use bigdecimal::BigDecimal;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use std::str;
use std::fs::OpenOptions;
pub struct ResultLogger;



impl ResultLogger {
    pub fn log(info: &str) {
        // Example: prepend log entries with a timestamp
        // println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), info);
    }

    // You can add more functions for different log levels if needed
    pub fn error(info: &str) {
        // eprintln!("[{}] Error: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), info);
    }
    
    /// Logs path results at /default_log_data/relay/date/input_amount/asset_time/
    pub fn log_results_default(result_log: Vec<PathNode>, start_node_name: String, input_amount: BigDecimal, relay: Relay){
        let json = serde_json::to_string_pretty(&result_log.clone()).unwrap();
        let relay_string = match relay {
            Relay::Polkadot => "polkadot",
            Relay::Kusama => "kusama"
        };

        // Get the current timestamp
        let timestamp = chrono::Local::now().format("%Y-%m-%d___%H-%M-%S").to_string();
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let time = chrono::Local::now().format("%H-%M-%S").to_string();
    
        // Construct the directory path for the current date
        let log_folder_path = format!("default_log_data/{}/{}/{}", relay_string, date, input_amount.to_string());
    
        // Create a directory for the current date if it doesn't exist
        match std::fs::create_dir_all(&log_folder_path) {
            Ok(_) => println!("Directory created successfully"),
            Err(e) => println!("Error creating directory: {:?}", e),
        }
    
        // Construct the file path including the directory
        let log_data_path = format!("{}/{}_{}.json", log_folder_path, start_node_name, time);
        println!("Log data path: {}", log_data_path);
       
        let mut file = File::create(log_data_path).expect("Failed to create file");
        file.write_all(json.as_bytes()).expect("Failed to write data");
    
        // let log_path = format!("result_log.txt", start_node.get_asset_name(), timestamp);
        let best_path_value = &result_log[result_log.len()-1].path_value;
        let result_log_string = format!("{} {} - {}", timestamp, start_node_name, best_path_value);
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("result_log.txt")
            .expect("Failed to open or create file");
        writeln!(file, "{}", result_log_string).expect("Failed to write data");
    
        // result_log.clone()
    }
    
    /// Logs results for TARGET search
    /// - Uses asset name, should be the asset name of the starting asset
    /// - /target_log_data/${relay}/${date}/${asset}_${time}
    pub fn log_results_target(path: Vec<PathNode>, asset_name: String, relay: Relay) {
        let relay_string = match relay {
            Relay::Kusama => "kusama",
            Relay::Polkadot => "polkadot"
        };
        let json = serde_json::to_string_pretty(&path.clone()).unwrap();

        // Get the current timestamp
        let timestamp = chrono::Local::now().format("%Y-%m-%d___%H-%M-%S").to_string();
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let time = chrono::Local::now().format("%H-%M-%S").to_string();
    
        // Construct the directory path for the current date
        let log_folder_path = format!("target_log_data/{}/{}", relay_string, date);
    
        // Create a directory for the current date if it doesn't exist
        match std::fs::create_dir_all(&log_folder_path) {
            Ok(_) => println!("Directory created successfully"),
            Err(e) => println!("Error creating directory: {:?}", e),
        }
    
        // Construct the file path including the directory
        let log_data_path = format!("{}/{}_{}.json", log_folder_path, asset_name.clone(), time);
        println!("Log data path: {}", log_data_path);

        // When creating the file, use the log_data_path which includes the directory
        let mut file = File::create(log_data_path).expect("Failed to create file");
        file.write_all(json.as_bytes()).expect("Failed to write data");
    
        // let log_path = format!("result_log.txt", start_node.get_asset_name(), timestamp);
        let best_path_value = &path[path.len()-1].path_value;
        let result_log_string = format!("{} {} - {}", timestamp, asset_name, best_path_value);
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("result_log.txt")
            .expect("Failed to open or create file");
        writeln!(file, "{}", result_log_string).expect("Failed to write data");
    }

    /// Logs results for FALLBACK search
    /// /fallback_log_data/relay/date/asset_name/time
    pub fn log_results_fallback(path: Vec<PathNode>, asset_name: String, relay: Relay) {
        let relay_string = match relay {
            Relay::Polkadot => "polkadot",
            Relay::Kusama => "kusama"
        };
        
        let json = serde_json::to_string_pretty(&path.clone()).unwrap();
        // Get the current timestamp
        let timestamp = chrono::Local::now().format("%Y-%m-%d___%H-%M-%S").to_string();
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let time = chrono::Local::now().format("%H-%M-%S").to_string();
    
        // Construct the directory path for the current date
        let log_folder_path = format!("fallback_log_data/{}/{}", relay_string.to_ascii_lowercase(), date);
    
        // Create a directory for the current date if it doesn't exist
        match std::fs::create_dir_all(&log_folder_path) {
            Ok(_) => println!("Directory created successfully"),
            Err(e) => println!("Error creating directory: {:?}", e),
        }
    
        // Construct the file path including the directory
        let log_data_path = format!("{}/{}_{}.json", log_folder_path, asset_name.clone(), time);
        println!("Log data path: {}", log_data_path);

        // When creating the file, use the log_data_path which includes the directory
        let mut file = File::create(log_data_path).expect("Failed to create file");
        file.write_all(json.as_bytes()).expect("Failed to write data");
    
        let best_path_value = &path[path.len()-1].path_value;
        let result_log_string = format!("{} {} - {}", timestamp, asset_name.clone(), best_path_value);
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("result_log.txt")
            .expect("Failed to open or create file");
        writeln!(file, "{}", result_log_string).expect("Failed to write data");
    
        // result_log.clone()
    }
    
}