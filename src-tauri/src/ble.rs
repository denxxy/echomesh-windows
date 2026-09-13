use std::sync::Arc;

use echomesh_core::transport::{decode_direct_packet, encode_direct_packet, NativeBleTransport};
use echomesh_core::DirectTransportCrypto;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct BleRuntime {
    session: Mutex<Option<BleSession>>,
}

struct BleSession {
    transport: Arc<NativeBleTransport>,
    crypto: Arc<DirectTransportCrypto>,
    receive_task: tokio::task::JoinHandle<()>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BleIncomingMessage {
    sender_peer_id: String,
    data: Vec<u8>,
}

#[tauri::command]
pub async fn ble_start(
    app: AppHandle,
    state: State<'_, BleRuntime>,
) -> Result<String, String> {
    let mut guard = state.session.lock().await;
    if let Some(session) = guard.as_ref() {
        return Ok(hex::encode(session.crypto.local_peer_id()));
    }

    let storage_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve BLE storage path: {e}"))?;
    std::fs::create_dir_all(&storage_path)
        .map_err(|e| format!("create BLE storage directory: {e}"))?;

    let crypto = DirectTransportCrypto::new(storage_path.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())?;
    let local_peer_id: [u8; 32] = crypto
        .local_peer_id()
        .as_slice()
        .try_into()
        .map_err(|_| "EchoMesh identity has invalid length".to_string())?;

    let transport = NativeBleTransport::start(local_peer_id)
        .await
        .map_err(|e| e.to_string())?;

    let receive_transport = transport.clone();
    let receive_crypto = crypto.clone();
    let receive_app = app.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(envelope) = receive_transport.recv_envelope().await {
            let packet = match encode_direct_packet(&envelope) {
                Ok(packet) => packet,
                Err(error) => {
                    tracing::warn!(error = %error, "discarding malformed BLE direct envelope");
                    continue;
                }
            };

            match receive_crypto.open(packet) {
                Ok(message) => {
                    let event = BleIncomingMessage {
                        sender_peer_id: hex::encode(message.sender_peer_id),
                        data: message.data,
                    };
                    if let Err(error) = receive_app.emit("echomesh://ble-message", event) {
                        tracing::warn!(error = %error, "failed to emit BLE message event");
                    }
                }
                Err(error) => {
                    tracing::warn!(error = %error, "rejected unauthenticated BLE packet");
                }
            }
        }
    });

    let peer_id_hex = hex::encode(local_peer_id);
    *guard = Some(BleSession {
        transport,
        crypto,
        receive_task,
    });
    Ok(peer_id_hex)
}

#[tauri::command]
pub async fn ble_send(
    peer_id_hex: String,
    data: Vec<u8>,
    state: State<'_, BleRuntime>,
) -> Result<(), String> {
    let peer_id = hex::decode(peer_id_hex.trim())
        .map_err(|_| "peer id must be 64 hexadecimal characters".to_string())?;
    let peer_id_array: [u8; 32] = peer_id
        .as_slice()
        .try_into()
        .map_err(|_| "peer id must decode to 32 bytes".to_string())?;

    let (transport, crypto) = {
        let guard = state.session.lock().await;
        let session = guard
            .as_ref()
            .ok_or_else(|| "BLE transport is not started".to_string())?;
        (session.transport.clone(), session.crypto.clone())
    };

    // DirectTransportCrypto deliberately returns a complete EMD1 packet. The
    // legacy native BLE primitive accepts the authenticated envelope and owns
    // EMD1 framing/EMB1 fragmentation, so unwrap exactly once at this boundary.
    let packet = crypto
        .seal(peer_id.to_vec(), data)
        .map_err(|e| e.to_string())?;
    let envelope = decode_direct_packet(&packet)
        .map_err(|e| e.to_string())?
        .to_vec();

    transport
        .send_envelope(peer_id_array, &envelope)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ble_stop(state: State<'_, BleRuntime>) -> Result<(), String> {
    let mut guard = state.session.lock().await;
    if let Some(session) = guard.take() {
        session.receive_task.abort();
        // Dropping the final transport handle stops the native GATT server via
        // NativeBleTransport::Drop and releases its advertisement resources.
        drop(session);
    }
    Ok(())
}
