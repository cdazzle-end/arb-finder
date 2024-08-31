use std::cell::RefCell;
use std::rc::Rc;
use std::hash::{Hasher, Hash};
use std::{path::Path, fs::File, io::Read};
use std::str;
use std::io;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde::de::{Deserializer, Error, Visitor};
use crate::asset_registry_2::{Asset, AssetLocation, MyAsset, TokenData};
use crate::constants;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relay {
    Polkadot,
    Kusama,
}

// #[derive(Debug, Deserialize)]
// struct TokenObject {
//     tokenData: TokenData,
//     hasLocation: bool,
//     tokenLocation: TokenLocation,
// }


pub fn get_asset_registry(relay: Relay) -> Vec<MyAsset> {
    let all_assets_file_location = match relay {
        Relay::Polkadot => format!("{}{}", constants::ASSET_REGISTRY_FOLDER, constants::ALL_POLKADOT_ASSETS),
        Relay::Kusama => format!("{}{}", constants::ASSET_REGISTRY_FOLDER, constants::ALL_KUSAMA_ASSETS)
    };
    let path: &Path = Path::new(&all_assets_file_location);
    let mut buf = vec![];
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut buf);
    
    let parsed_assets: Vec<MyAsset> = serde_json::from_str(str::from_utf8(&buf).unwrap()).unwrap();
    parsed_assets
}

pub fn get_asset_by_asset_key(asset_key: String, relay: Relay) -> MyAsset {
    let asset_registry = get_asset_registry(relay);

    let matching_asset: Option<MyAsset> = asset_registry
    .into_iter()
    .find(|asset| {
        let asset_registry_key = get_asset_key(asset.clone());
        asset_registry_key == asset_key
    });

    matching_asset.unwrap()
}

pub fn get_xcm_assets(chain_id: usize, asset_id: &str, relay: Relay) -> Option<MyAsset>{
    let asset_registry: Vec<MyAsset> = get_asset_registry(relay);
    let mut asset_map_by_location: HashMap<AssetLocation, Vec<MyAsset>> = HashMap::new();

    for asset in asset_registry.clone() {
        if asset.hasLocation {
            let asset_location: AssetLocation = parse_asset_location(&asset).unwrap();

            asset_map_by_location.entry(asset_location)
            .or_insert(Vec::new())
            .push(asset)
        }
    }

    let chain_assets: Vec<MyAsset> = asset_registry
        .into_iter()
        .filter(|asset| {
            match asset.tokenData.clone() {
                TokenData::MyAsset(asset_data) => asset_data.chain == (chain_id as u64),
                _ => false
            }
        })
        .collect();

    println!("Searching for ID: {}", asset_id);

    let input_asset_id_value: Value = serde_json::from_str(asset_id).unwrap_or(Value::Null);

    let matching_asset = chain_assets.into_iter().find(|asset| {
        match &asset.tokenData {
            TokenData::MyAsset(asset_data) => asset_data.localId == input_asset_id_value,
            _ => false
        }
    });

    let asset_location: AssetLocation = parse_asset_location(&matching_asset.clone().unwrap()).unwrap();
    
    // Output the groups
    for (location, group) in asset_map_by_location.iter() {
        println!("Location: {:?}, Tokens: {:?}", location, group);
    }


    println!("{:?}", asset_location);

    let all_assets_at_location = asset_map_by_location.get(&asset_location);

    println!("All assets at location: ");
    for asset in all_assets_at_location.unwrap() {
        println!("{:#?}", asset);
    }

    matching_asset
}

pub fn get_assets_at_location(asset: MyAsset, relay: Relay) -> Vec<MyAsset> {
    let asset_registry: Vec<MyAsset> = get_asset_registry(relay);
    let mut asset_map_by_location: HashMap<AssetLocation, Vec<MyAsset>> = HashMap::new();

    for asset in asset_registry.clone() {
        if asset.hasLocation {
            let asset_location: AssetLocation = parse_asset_location(&asset).unwrap();

            asset_map_by_location.entry(asset_location)
            .or_insert(Vec::new())
            .push(asset)
        }
    }

    let asset_location: AssetLocation = parse_asset_location(&asset.clone()).unwrap();

    let all_assets_at_location = asset_map_by_location.get(&asset_location).unwrap();

    println!("All assets at location: ");
    for asset in all_assets_at_location.clone() {
        println!("{:#?}", asset);
    }

    all_assets_at_location.clone()
}

pub fn get_asset_by_chain_and_id(chain_id: usize, asset_id: &str, relay: Relay) -> MyAsset {
    let asset_registry: Vec<MyAsset> = get_asset_registry(relay);

    let input_asset_id_value: Value = serde_json::from_str(asset_id).unwrap_or(Value::Null);

    let matching_asset: Option<MyAsset> = asset_registry
        .into_iter()
        .find(|asset| {
            match asset.tokenData.clone() {
                TokenData::MyAsset(asset_data) => asset_data.chain == (chain_id as u64) && asset_data.localId == input_asset_id_value,
                _ => false
            }
        });


    println!("Get asset: {} | {}", chain_id, asset_id);
    println!("Matching asset: {:#?}", matching_asset);

    matching_asset.unwrap()
}

pub fn get_asset_key(asset: MyAsset) -> String {
    let asset_key = match asset.tokenData {
        TokenData::MyAsset(data) => data.chain.to_string() + &data.localId.to_string(),
        TokenData::CexAsset(data) => data.exchange.to_string() + &data.assetTicker.to_string()
    };
    return asset_key
}

fn parse_asset_location(parsed_asset_registry_object: &MyAsset) -> Option<AssetLocation> {
    match &parsed_asset_registry_object.tokenLocation {
        Some(location) if location.is_string() => Some(AssetLocation::new(true, None, None)),
        Some(location) if location.is_object() => {
            let location_obj = location.as_object().unwrap();
            let xtype = location_obj.keys().next().map(|x| x.to_string());
            let properties = location_obj.get(xtype.as_ref().unwrap()).unwrap();
            // println!("{:?}", properties);
            let properties = match properties{
                x if x.is_array() => x.as_array().unwrap().iter().map(|x| serde_json::to_string(x.as_object().unwrap()).unwrap()).collect(),
                x => vec![serde_json::to_string(x.as_object().unwrap()).unwrap()]
            };
            Some(AssetLocation::new(false, xtype, Some(properties)))
        },
        _ => None,
    }
}