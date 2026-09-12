use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, State};

use echomesh_core::EchoMeshClient;
use crate::state::{AppState, RelayInfo, RelayStatusDto, TauriEventsListener};

#[tauri::command]
pub async fn connect_to_relay(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    host: String,
    port: u16,
    public_key_hex: String,
    secret_token_hex: Option<String>,
) -> Result<(), String> {
    let clean_key = public_key_hex.trim().trim_start_matches("0x").to_string();
    
    // Parse public key: support hex (64 chars) or base64 (44 chars)
    let public_key_bytes: Vec<u8> = if clean_key.len() == 64 {
        hex::decode(&clean_key).map_err(|e| format!("Invalid hex public key: {}", e))?
    } else {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&clean_key)
            .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&clean_key))
            .map_err(|e| format!("Invalid public key (not 64-char hex or base64): {}", e))?
    };

    if public_key_bytes.len() != 32 {
        return Err(format!(
            "Public key must be 32 bytes, got {}",
            public_key_bytes.len()
        ));
    }

    let relay_addr = format!("{}:{}", host.trim(), port);

    // Disconnect previous client if existing
    {
        let client_guard = state.client.read().await;
        if let Some(ref client) = *client_guard {
            let _ = client.disconnect();
        }
    }

    // Initialize or obtain EchoMeshClient
    let client_arc = {
        let mut client_guard = state.client.write().await;
        if client_guard.is_none() {
            let storage_str = state.storage_path.to_string_lossy().to_string();
            let status_ref = Arc::new(std::sync::atomic::AtomicU8::new(0));

            let listener = Box::new(TauriEventsListener {
                app_handle: app_handle.clone(),
                status_ref: status_ref.clone(),
            });

            let client = EchoMeshClient::new(storage_str, listener)
                .map_err(|e| format!("Failed to create EchoMeshClient: {:?}", e))?;
            *client_guard = Some(client.clone());
            client
        } else {
            client_guard.as_ref().unwrap().clone()
        }
    };

    // Update current relay configuration
    {
        let mut relay_guard = state.current_relay.write().await;
        *relay_guard = Some(RelayInfo {
            host: host.clone(),
            port,
            public_key_hex: hex::encode(&public_key_bytes),
            secret_token_hex: secret_token_hex.clone(),
        });
    }

    // Connect to relay
    client_arc
        .connect(relay_addr, public_key_bytes, secret_token_hex)
        .map_err(|e| format!("Connection error: {:?}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    let client_guard = state.client.read().await;
    if let Some(ref client) = *client_guard {
        client.disconnect().map_err(|e| format!("Disconnect error: {:?}", e))?;
    }
    state.connection_status.store(0, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<RelayStatusDto, String> {
    let status_code = state.connection_status.load(Ordering::SeqCst);
    let status_text = match status_code {
        0 => "Offline",
        1 => "Connecting",
        2 => "Connected",
        3 => "BLE Mesh Fallback",
        _ => "Unknown",
    }
    .to_string();

    let client_guard = state.client.read().await;
    let ping_ms = client_guard.as_ref().map(|c| c.ping_ms()).unwrap_or(0);

    let relay_guard = state.current_relay.read().await;
    let (host, port, public_key_hex) = if let Some(ref r) = *relay_guard {
        (r.host.clone(), r.port, r.public_key_hex.clone())
    } else {
        ("77.81.5.109".to_string(), 8443, "a19be0e77828448ddf354673e55c0ae97cc523d28891db96bba81866c9acd606".to_string())
    };

    Ok(RelayStatusDto {
        connection_status: status_code,
        status_text,
        ping_ms,
        host,
        port,
        public_key_hex,
    })
}
