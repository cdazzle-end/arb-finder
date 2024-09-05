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
use crate::adjacency_table_2::AdjacencyTable2;
use crate::asset_registry_2::{Asset, AssetLocation, AssetRegistry2, MyAsset, TokenData};
use crate::{constants, NodePath};
use crate::liq_pool_registry_2::LiqPoolRegistry2;
use crate::token_graph_2::{PathNode, TokenGraph2};
use std::str::FromStr;

type AssetPointer = Rc<RefCell<Asset>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relay {
    Polkadot,
    Kusama,
}
impl Relay {

    pub fn from_str(s: &str) -> Relay {
        match s.to_lowercase().as_str() {
            "polkadot" => Relay::Polkadot,
            "kusama" => Relay::Kusama,
            _ => panic!("Invalid network: {}. Must be 'polkadot' or 'kusama'.", s),
        }
    }
}

// #[derive(Debug, Deserialize)]
// struct TokenObject {
//     tokenData: TokenData,
//     hasLocation: bool,
//     tokenLocation: TokenLocation,
// }
pub fn build_asset_registry_old(relay: Relay) -> AssetRegistry2 {
    match relay {
        Relay::Polkadot => AssetRegistry2::build_asset_registry_polkadot(relay),
        Relay::Kusama => AssetRegistry2::build_asset_registry()
    }
}


/// Build asset registry from data base
/// - Ignore list will remove location property from asset
/// - The asset can still be traded, just as if it was a non xcm token
pub fn build_asset_registry(relay: Relay) -> AssetRegistry2 {
    let all_assets_file_location = match relay {
        Relay::Polkadot => format!("{}{}", constants::ASSET_REGISTRY_FOLDER, constants::ALL_POLKADOT_ASSETS),
        Relay::Kusama => format!("{}{}", constants::ASSET_REGISTRY_FOLDER, constants::ALL_KUSAMA_ASSETS)
    };
    let asset_file_path: &Path = Path::new(&all_assets_file_location);
    let mut asset_file_buffer = vec![];
    let mut asset_file = File::open(asset_file_path).unwrap();
    asset_file.read_to_end(&mut asset_file_buffer);

    let parsed_assets: Vec<MyAsset> = serde_json::from_str(str::from_utf8(&asset_file_buffer).unwrap()).unwrap();
    println!("Number of parsed assets: {}", parsed_assets.len());

    let ignore_file_path = constants::ASSET_IGNORE_LIST;
    let mut ignore_file_buffer = vec![];
    let mut ignore_file = File::open(ignore_file_path).unwrap();
    ignore_file.read_to_end(&mut ignore_file_buffer).unwrap();
    let parsed_ignore_file: Value = serde_json::from_str(str::from_utf8(&ignore_file_buffer).unwrap()).unwrap();
    let ignore_list_assets: Vec<MyAsset> = serde_json::from_value(parsed_ignore_file).unwrap();
    let ignore_list_locations: Vec<String> = ignore_list_assets.clone().into_iter().map(|asset| {
        let ignore_asset_location = parse_asset_location(&asset);
        let ignore_asset = Rc::new(RefCell::new(Asset::new(asset.tokenData.clone(), ignore_asset_location)));
        let location_string = ignore_asset.borrow().get_asset_location_string().clone();
        location_string
    }).collect();
    
    let ignore_list_asset_keys: Vec<String> = ignore_list_assets.into_iter().map(|asset| {
        let ignore_asset = Rc::new(RefCell::new(Asset::new(asset.tokenData.clone(), None)));
        let map_key = ignore_asset.borrow().get_map_key();
        map_key
    }).collect();

    let mut asset_map: HashMap<String, Vec<AssetPointer>> = HashMap::new();
    let mut asset_location_map: HashMap<AssetLocation, Vec<AssetPointer>> = HashMap::new();

    for asset in parsed_assets{
        let asset_location = parse_asset_location(&asset);
        let new_asset = Rc::new(RefCell::new(Asset::new(asset.tokenData, asset_location)));
        let map_key = new_asset.borrow().get_map_key();
        if ignore_list_asset_keys.contains(&map_key){
            println!("Ignoring asset: {}", map_key);

            // Remove asset location so it wont be added to xcm adjacent nodes
            new_asset.borrow_mut().asset_location = None;
        }
        asset_map.entry(map_key).or_insert(vec![]).push(new_asset.clone());

        if let Some(location) = new_asset.borrow().asset_location.clone() {
            asset_location_map.entry(location).or_insert(vec![]).push(new_asset.clone());
        };
    }

    let asset_registry: AssetRegistry2 = AssetRegistry2 {
        asset_map: asset_map,
        asset_location_map: asset_location_map,
    };

    asset_registry
}

pub fn build_liq_pool_registry(asset_registry: &AssetRegistry2, relay: Relay) -> LiqPoolRegistry2 {
    match relay {
        Relay::Kusama => LiqPoolRegistry2::build_liqpool_registry_kusama(&asset_registry),
        Relay::Polkadot => LiqPoolRegistry2::build_liqpool_registry_polkadot(&asset_registry)
    }
}

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

    // println!("Searching for ID: {}", asset_id);

    let input_asset_id_value: Value = serde_json::from_str(asset_id).unwrap_or(Value::Null);

    let matching_asset = chain_assets.into_iter().find(|asset| {
        match &asset.tokenData {
            TokenData::MyAsset(asset_data) => asset_data.localId == input_asset_id_value,
            _ => false
        }
    });

    let asset_location: AssetLocation = parse_asset_location(&matching_asset.clone().unwrap()).unwrap();
    
    // Output the groups
    // for (location, group) in asset_map_by_location.iter() {
    //     println!("Location: {:?}, Tokens: {:?}", location, group);
    // }


    // println!("{:?}", asset_location);

    let all_assets_at_location = asset_map_by_location.get(&asset_location);

    // println!("All assets at location: ");
    // for asset in all_assets_at_location.unwrap() {
    //     println!("{:#?}", asset);
    // }

    matching_asset
}

/// Search for all assets in the asset registry that have the same location as the input MyAsset
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

    // println!("All assets at location: ");
    // for asset in all_assets_at_location.clone() {
    //     println!("{:#?}", asset);
    // }

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


    // println!("Get asset: {} | {}", chain_id, asset_id);
    // println!("Matching asset: {:#?}", matching_asset);

    matching_asset.unwrap()
}

pub fn get_asset_key(asset: MyAsset) -> String {
    let asset_key = match asset.tokenData {
        TokenData::MyAsset(data) => data.chain.to_string() + &data.localId.to_string(),
        TokenData::CexAsset(data) => data.exchange.to_string() + &data.assetTicker.to_string()
    };
    return asset_key
}

/// Build asset registry, liq pool registry, adjacency list, and token graph for given relay
pub fn build_token_graph(relay: Relay) -> TokenGraph2{
    let mut asset_registry: AssetRegistry2 = build_asset_registry(relay);

    // asset_registry.display_all_assets();

    println!("Created asset registry");

    let lp_registry: LiqPoolRegistry2 = build_liq_pool_registry(&asset_registry, relay);

    println!("Created lp registry");

    // lp_registry.display_liq_pools();

    let adjacency_list = AdjacencyTable2::build_table_2(&lp_registry);
    
    let graph: TokenGraph2 = TokenGraph2::build_graph_2(asset_registry, lp_registry, adjacency_list);
    graph
    
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


/// Take path results, Vec<NodePointers> from arb finder, format it to Vec<PathNode> for result logs
pub fn return_path_nodes(path: NodePath) -> Vec<PathNode> {
    let target_node = path[path.len() - 1].borrow();
    let path_values = &target_node.path_values;
    let path_value_types = &target_node.path_value_types;
    let path_datas = &target_node.path_datas;
    let mut result_log: Vec<PathNode> = Vec::new();
    for(i, node) in path.iter().enumerate(){
        let path_node = PathNode{
            node_key: node.borrow().get_asset_key(),
            asset_name: node.borrow().get_asset_name(),
            path_value: path_values[i].to_string(),
            path_type: path_value_types[i].clone(),
            path_data: path_datas[i].clone(),
        };
        result_log.push(path_node);
    }
    result_log.clone()

}
