#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kaspa_graffiti::commands::{generate_wallet, load_wallet, validate_address, get_balance, get_utxos, transfer};
use serde_json;

#[tauri::command]
async fn wallet_generate() -> Result<String, String> {
    match generate_wallet().await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn wallet_load(private_key: &str) -> Result<String, String> {
    match load_wallet(private_key).await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn address_validate(address: &str) -> Result<bool, String> {
    Ok(validate_address(address).await)
}

#[tauri::command]
async fn balance_get(address: &str, rpc_url: Option<&str>) -> Result<String, String> {
    match get_balance(address, rpc_url).await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn utxos_get(address: &str, rpc_url: Option<&str>) -> Result<String, String> {
    match get_utxos(address, rpc_url).await {
        Ok(utxos) => serde_json::to_string(&utxos).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn wallet_hd_generate() -> Result<String, String> {
    use kaspa_graffiti::commands::generate_hd_wallet;
    match generate_hd_wallet().await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn wallet_hd_load(seed: &str) -> Result<String, String> {
    use kaspa_graffiti::commands::load_hd_wallet;
    match load_hd_wallet(seed).await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn derive_address(seed: &str, index: u32, change: Option<bool>) -> Result<String, String> {
    use kaspa_graffiti::commands::derive_address_from_seed;
    match derive_address_from_seed(seed, index, change.unwrap_or(false)).await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn derive_many(private_key: &str, count: u32) -> Result<String, String> {
    use kaspa_graffiti::commands::derive_many_addresses;
    match derive_many_addresses(private_key, count, false).await {
        Ok(info) => serde_json::to_string(&info).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn wallet_transfer(private_key: &str, recipient: &str, amount: u64, rpc_url: Option<&str>) -> Result<String, String> {
    match transfer(private_key, recipient, amount, rpc_url).await {
        Ok(result) => serde_json::to_string(&result).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            wallet_generate,
            wallet_load,
            address_validate,
            balance_get,
            utxos_get,
            wallet_hd_generate,
            wallet_hd_load,
            derive_address,
            derive_many,
            wallet_transfer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
