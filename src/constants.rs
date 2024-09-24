// Using this file to track local file location constants

pub const ASSET_REGISTRY_FOLDER: &str = "C:/Users/dazzl/CodingProjects/substrate/polkadot_asset_registry/asset_registry/";
pub const ASSET_IGNORE_LIST: &str = "C:/Users/dazzl/CodingProjects/substrate/polkadot_asset_registry/asset_registry/ignore_list.json";
pub const LP_REGISTRY_FOLDER: &str = "C:/Users/dazzl/CodingProjects/substrate/polkadot_asset_registry/lp_registry/";
pub const XCM_FEE_BOOK: &str = "C:/Users/dazzl/CodingProjects/substrate/xcm-test/data/newEventFeeBook.json";
pub const ALL_POLKADOT_ASSETS: &str = "allAssetsPolkadotCollected.json";
pub const ALL_KUSAMA_ASSETS: &str = "allAssetsKusamaCollected.json";
// pub const POLKADOT_ASSETS_FILE: &str = format!("{}{}", &ASSET_REGISTRY_FOLDER, &ALL_POLKADOT_ASSETS).as_str();

pub const KUSAMA_ASSET_CHAINS: &'static [&'static str] = &["aca", "bnc_polkadot", "glmr", "hdx", "para", "other_polkadot", "glmr_evm", "asset_hub_polkadot"];
pub const POLKADOT_ASSET_CHAINS: &'static [&'static str] = &["aca", "bnc_polkadot", "glmr", "hdx", "para", "other_polkadot", "glmr_evm", "asset_hub_polkadot"];

pub const POLKADOT_START_NODE: &str = "2000{\"NativeAssetId\":{\"Token\":\"DOT\"}}"; // Acala node
pub const KUSAMA_START_NODE: &str = "2000{\"NativeAssetId\":{\"Token\":\"KSM\"}}"; // Karura node

pub const MOONBEAM_IGNORE_LIST: &'static [&'static str] = &[
    "0x54184eabc2a13830931601cc31c391c119784e3d", // MyTradeLp
];