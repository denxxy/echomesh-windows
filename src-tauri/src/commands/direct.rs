use tauri::{AppHandle, State};

use crate::commands::chat::ensure_client;
use crate::state::AppState;

#[tauri::command]
pub async fn start_ble_mesh(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = ensure_client(&app_handle, &state).await?;
    state.ensure_ble(client).await.map(|_| ())
}

/// Sends opaque application bytes through the same client-to-client E2EE
/// envelope used by LAN/relay, then transports that envelope over native GATT.
#[tauri::command]
pub async fn send_ble_packet(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    recipient_id_hex: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let clean = recipient_id_hex.trim().trim_start_matches("0x");
    let recipient_vec = hex::decode(clean)
        .map_err(|_| "Recipient peer ID must be valid hex".to_string())?;
    let recipient: [u8; 32] = recipient_vec
        .as_slice()
        .try_into()
        .map_err(|_| "Recipient peer ID must be exactly 32 bytes".to_string())?;

    let client = ensure_client(&app_handle, &state).await?;
    let direct_packet = client
        .build_direct_packet(recipient_vec, data)
        .map_err(|e| format!("Failed to build E2EE direct packet: {e}"))?;
    let envelope = echomesh_core::transport::decode_direct_packet(&direct_packet)
        .map_err(|e| format!("Invalid direct packet: {e}"))?
        .to_vec();

    let ble = state.ensure_ble(client).await?;
    ble.send_envelope(recipient, &envelope)
        .await
        .map_err(|e| format!("BLE send failed: {e}"))
}
