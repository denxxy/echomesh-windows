use tauri::{AppHandle, State};

use crate::state::AppState;
use super::chat::ensure_client;

#[tauri::command]
pub async fn start_ble_mesh(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = ensure_client(&app_handle, &state).await?;
    state.ensure_ble(client).await?;
    Ok(())
}

#[tauri::command]
pub async fn send_ble_packet(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    recipient_peer_id_hex: String,
    payload: Vec<u8>,
) -> Result<(), String> {
    let clean = recipient_peer_id_hex.trim().trim_start_matches("0x");
    let recipient_vec = hex::decode(clean)
        .map_err(|_| "Recipient peer ID must be valid hex".to_string())?;
    let recipient: [u8; 32] = recipient_vec
        .as_slice()
        .try_into()
        .map_err(|_| "Recipient peer ID must be exactly 32 bytes".to_string())?;

    let client = ensure_client(&app_handle, &state).await?;
    let transport = state.ensure_ble(client.clone()).await?;

    // The core constructs the canonical EMD1 packet whose payload is an EME1
    // authenticated ciphertext. BLE is only a bearer and must not unwrap or
    // decrypt that packet before GATT transmission.
    let direct_packet = client
        .build_direct_packet(recipient.to_vec(), payload)
        .map_err(|e| format!("Failed to build encrypted BLE packet: {e}"))?;

    transport
        .send_packet(recipient, &direct_packet)
        .await
        .map_err(|e| format!("Failed to send BLE packet: {e}"))
}
