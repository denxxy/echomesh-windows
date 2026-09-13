use std::sync::Arc;

use tauri::{AppHandle, State};

use echomesh_core::transport::decode_direct_packet;
use echomesh_core::EchoMeshClient;

use crate::state::{AppState, TauriEventsListener};

async fn ensure_client(
    app_handle: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<Arc<EchoMeshClient>, String> {
    let mut guard = state.client.write().await;
    if let Some(client) = guard.as_ref().cloned() {
        return Ok(client);
    }

    let listener = Box::new(TauriEventsListener {
        app_handle: app_handle.clone(),
        status_ref: Arc::new(std::sync::atomic::AtomicU8::new(0)),
    });
    let client = EchoMeshClient::new(
        state.storage_path.to_string_lossy().to_string(),
        listener,
    )
    .map_err(|e| format!("Failed to create EchoMeshClient: {e}"))?;
    *guard = Some(client.clone());
    Ok(client)
}

/// Starts the Windows native BLE central+peripheral bearer and the background
/// receive loop. Calling this repeatedly is idempotent.
#[tauri::command]
pub async fn start_ble_transport(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let client = ensure_client(&app_handle, &state).await?;
    state.ensure_ble(client).await?;
    Ok(())
}

/// Sends an arbitrary application payload to a 32-byte EchoMesh peer over BLE.
/// The native GATT layer sees only an opaque E2EE envelope; identity keys stay
/// inside echomesh-core.
#[tauri::command]
pub async fn send_ble_packet(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    recipient_id_hex: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let clean = recipient_id_hex.trim().trim_start_matches("0x");
    let recipient_vec = hex::decode(clean)
        .map_err(|_| "Recipient peer ID must be a 64-character hex string".to_string())?;
    let recipient: [u8; 32] = recipient_vec
        .as_slice()
        .try_into()
        .map_err(|_| "Recipient peer ID must decode to 32 bytes".to_string())?;

    let client = ensure_client(&app_handle, &state).await?;
    let transport = state.ensure_ble(client.clone()).await?;
    let packet = client
        .build_direct_packet(recipient.to_vec(), data)
        .map_err(|e| format!("Failed to encrypt BLE payload: {e}"))?;
    let envelope = decode_direct_packet(&packet)
        .map_err(|e| format!("Failed to prepare BLE packet: {e}"))?;

    transport
        .send_envelope(recipient, envelope)
        .await
        .map_err(|e| format!("BLE delivery failed: {e}"))
}
