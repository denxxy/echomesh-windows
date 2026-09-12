use std::sync::Arc;
use tauri::{AppHandle, State};

use echomesh_core::protocol::ECHO_SERVICE_PEER_ID;
use echomesh_core::EchoMeshClient;
use crate::state::{AppState, MessageDto, TauriEventsListener};

async fn ensure_client(app_handle: &AppHandle, state: &State<'_, AppState>) -> Result<Arc<EchoMeshClient>, String> {
    let mut client_guard = state.client.write().await;
    if client_guard.is_none() {
        let storage_str = state.storage_path.to_string_lossy().to_string();
        let status_ref = Arc::new(std::sync::atomic::AtomicU8::new(0));

        let listener = Box::new(TauriEventsListener {
            app_handle: app_handle.clone(),
            status_ref,
        });

        let client = EchoMeshClient::new(storage_str, listener)
            .map_err(|e| format!("Failed to create EchoMeshClient: {:?}", e))?;
        *client_guard = Some(client.clone());
        Ok(client)
    } else {
        Ok(client_guard.as_ref().unwrap().clone())
    }
}

#[tauri::command]
pub async fn send_message(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    recipient_id_hex: String,
    text: String,
) -> Result<MessageDto, String> {
    if text.trim().is_empty() {
        return Err("Message text cannot be empty".to_string());
    }

    let target_hex = if recipient_id_hex.trim().is_empty()
        || recipient_id_hex.to_lowercase().contains("echo")
        || recipient_id_hex.trim() == hex::encode(ECHO_SERVICE_PEER_ID)
    {
        hex::encode(ECHO_SERVICE_PEER_ID)
    } else {
        let clean = recipient_id_hex.trim().trim_start_matches("0x");
        if clean.len() != 64 {
            return Err("Recipient peer ID must be a 64-character hex string or echo node".to_string());
        }
        clean.to_string()
    };

    let client = ensure_client(&app_handle, &state).await?;
    let record = client
        .send_chat_message(target_hex, text)
        .map_err(|e| format!("Failed to send chat message: {:?}", e))?;

    Ok(MessageDto::from(record))
}

#[tauri::command]
pub async fn get_history(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    peer_id_hex: String,
    limit: Option<u32>,
) -> Result<Vec<MessageDto>, String> {
    let target_hex = if peer_id_hex.trim().is_empty()
        || peer_id_hex.to_lowercase().contains("echo")
        || peer_id_hex.trim() == hex::encode(ECHO_SERVICE_PEER_ID)
    {
        hex::encode(ECHO_SERVICE_PEER_ID)
    } else {
        peer_id_hex.trim().trim_start_matches("0x").to_string()
    };

    let client = ensure_client(&app_handle, &state).await?;
    let records = client
        .get_messages(target_hex, limit.unwrap_or(100))
        .map_err(|e| format!("Failed to get history: {:?}", e))?;

    Ok(records.into_iter().map(MessageDto::from).collect())
}
