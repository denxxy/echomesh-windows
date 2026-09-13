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
    state
        .connection_status
        .store(4, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn send_ble_packet(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    recipient_peer_id_hex: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let clean = recipient_peer_id_hex.trim().trim_start_matches("0x");
    let recipient = hex::decode(clean)
        .map_err(|_| "Recipient peer ID must be a 64-character hex string".to_string())?;
    let recipient_peer_id: [u8; 32] = recipient
        .as_slice()
        .try_into()
        .map_err(|_| "Recipient peer ID must decode to exactly 32 bytes".to_string())?;

    let client = ensure_client(&app_handle, &state).await?;
    let packet = client
        .build_direct_packet(recipient, data)
        .map_err(|e| format!("Failed to build E2EE BLE packet: {e}"))?;
    let transport = state.ensure_ble(client).await?;
    transport
        .send_packet(recipient_peer_id, &packet)
        .await
        .map_err(|e| format!("Failed to send BLE packet: {e}"))?;
    state
        .connection_status
        .store(4, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}
