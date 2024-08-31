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
use crate::asset_registry_2::{Asset, AssetLocation, MyAsset};
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


pub fn get_asset_registry(relay: Relay){

    let chains = vec!["aca", "bnc_polkadot", "glmr", "hdx", "para", "other_polkadot", "glmr_evm", "asset_hub_polkadot"];
    let parsed_files = chains
        .into_iter()
        .map(|chain| {
            let path_string: String = format!("{}{}_assets.json", constants::ASSET_REGISTRY_FOLDER, chain);
            // println!("path_string: {}", path_string);
            let path = Path::new(&path_string);
            let mut buf = vec![];
            let mut file = File::open(path)?;
            file.read_to_end(&mut buf)?;
            let parsed = serde_json::from_str(str::from_utf8(&buf).unwrap())?;
            Ok(parsed)
        })
        .collect::<Result<Vec<Value>, io::Error>>()
        .unwrap();

    let all_assets_file_location = match relay {
        Relay::Polkadot => format!("{}{}", constants::ASSET_REGISTRY_FOLDER, constants::ALL_POLKADOT_ASSETS),
        Relay::Kusama => format!("{}{}", constants::ASSET_REGISTRY_FOLDER, constants::ALL_KUSAMA_ASSETS)
    };
    let path: &Path = Path::new(&all_assets_file_location);
    let mut buf = vec![];
    let mut file = File::open(path).unwrap();
    file.read_to_end(&mut buf);
    // let parsed: Value = serde_json::from_str(str::from_utf8(&buf).unwrap()).unwrap();
    
    let parsed_assets: Vec<MyAsset> = serde_json::from_str(str::from_utf8(&buf).unwrap()).unwrap();

    println!("{:?}", parsed_assets);

    // let mut asset_map: HashMap<String, Vec<AssetPointer>> = HashMap::new();
    // let mut asset_location_map: HashMap<AssetLocation, Vec<AssetPointer>> = HashMap::new();

    // let asset_array: Vec<MyAsset> = serde_json::from_value(parsed_assets).unwrap();
    // for asset in asset_array{
    //     let asset_location = parse_asset_location(&asset);
    //     // println!("{:?}", asset.tokenData);
    //     let mut new_asset = Rc::new(RefCell::new(Asset::new(asset.tokenData, asset_location)));
    //     let map_key = new_asset.borrow().get_map_key();

    //     asset_map.entry(map_key).or_insert(vec![]).push(new_asset.clone());

    //     if let Some(location) = new_asset.borrow().asset_location.clone() {
    //         asset_location_map.entry(location).or_insert(vec![]).push(new_asset.clone());
    //     };
    // }


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