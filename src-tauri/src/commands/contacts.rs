use std::sync::Arc;
use tauri::{AppHandle, State};

use echomesh_core::EchoMeshClient;
use crate::state::{AppState, ContactDto, TauriEventsListener};

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
pub async fn get_contacts(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ContactDto>, String> {
    let client = ensure_client(&app_handle, &state).await?;
    let contacts = client.get_contacts().map_err(|e| format!("{:?}", e))?;
    Ok(contacts.into_iter().map(ContactDto::from).collect())
}

#[tauri::command]
pub async fn add_contact(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    peer_id_hex: String,
    name: String,
) -> Result<ContactDto, String> {
    let clean_hex = peer_id_hex.trim().trim_start_matches("0x");
    if clean_hex.len() != 64 || !clean_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Peer ID must be a 64-character hexadecimal string (32 bytes)".to_string());
    }

    let client = ensure_client(&app_handle, &state).await?;
    client
        .add_contact(clean_hex.to_string(), name.trim().to_string())
        .map_err(|e| format!("{:?}", e))?;

    let peer_bytes = hex::decode(clean_hex).unwrap();
    let is_echo_node = peer_bytes == echomesh_core::protocol::ECHO_SERVICE_PEER_ID;

    Ok(ContactDto {
        peer_id: clean_hex.to_string(),
        name: name.trim().to_string(),
        added_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        is_echo_node,
    })
}
