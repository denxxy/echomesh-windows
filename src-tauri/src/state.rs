use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::RwLock;

use echomesh_core::{
    Contact, CoreEventsListener, DeliveryStatus, EchoMeshClient, MessageRecord, NetworkState,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayInfo {
    pub host: String,
    pub port: u16,
    pub public_key_hex: String,
    pub secret_token_hex: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageDto {
    pub id: String,
    pub conversation_peer_id: String,
    pub sender_peer_id: String,
    pub text: String,
    pub timestamp: u64,
    pub is_outgoing: bool,
    pub status: u8, // 0 = Sending, 1 = Sent, 2 = Failed
}

impl From<MessageRecord> for MessageDto {
    fn from(r: MessageRecord) -> Self {
        Self {
            id: r.id,
            conversation_peer_id: hex::encode(&r.conversation_peer_id),
            sender_peer_id: hex::encode(&r.sender_peer_id),
            text: r.text,
            timestamp: r.timestamp,
            is_outgoing: r.is_outgoing,
            status: r.status,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContactDto {
    pub peer_id: String,
    pub name: String,
    pub added_at: u64,
    pub is_echo_node: bool,
}

impl From<Contact> for ContactDto {
    fn from(c: Contact) -> Self {
        let is_echo_node = c.peer_id == echomesh_core::protocol::ECHO_SERVICE_PEER_ID;
        Self {
            peer_id: hex::encode(&c.peer_id),
            name: c.name,
            added_at: c.added_at,
            is_echo_node,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayStatusDto {
    pub connection_status: u8,
    pub status_text: String,
    pub ping_ms: u32,
    pub host: String,
    pub port: u16,
    pub public_key_hex: String,
}

/// Global Application State kept in Tauri
pub struct AppState {
    pub client: RwLock<Option<Arc<EchoMeshClient>>>,
    pub connection_status: AtomicU8,
    pub current_relay: RwLock<Option<RelayInfo>>,
    pub storage_path: PathBuf,
}

impl AppState {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            client: RwLock::new(None),
            connection_status: AtomicU8::new(0), // 0 = Offline
            current_relay: RwLock::new(None),
            storage_path,
        }
    }

    pub fn set_status(&self, status: NetworkState) -> u8 {
        let code = match status {
            NetworkState::Offline => 0,
            NetworkState::Connecting => 1,
            NetworkState::ConnectedRealityRelay => 2,
            NetworkState::ConnectedBleMeshFallback => 3,
        };
        self.connection_status.store(code, Ordering::SeqCst);
        code
    }
}

/// Bridge listener for core events that pushes Tauri IPC events to the UI.
pub struct TauriEventsListener {
    pub app_handle: AppHandle,
    pub status_ref: Arc<AtomicU8>,
}

impl CoreEventsListener for TauriEventsListener {
    fn on_state_changed(&self, state: NetworkState) {
        let code = match state {
            NetworkState::Offline => 0,
            NetworkState::Connecting => 1,
            NetworkState::ConnectedRealityRelay => 2,
            NetworkState::ConnectedBleMeshFallback => 3,
        };
        self.status_ref.store(code, Ordering::SeqCst);

        let status_text = match state {
            NetworkState::Offline => "Offline",
            NetworkState::Connecting => "Connecting",
            NetworkState::ConnectedRealityRelay => "Connected",
            NetworkState::ConnectedBleMeshFallback => "BLE Mesh",
        };

        let _ = self.app_handle.emit(
            "connection-status-changed",
            serde_json::json!({
                "status": code,
                "status_text": status_text,
            }),
        );
    }

    fn on_message_received(&self, message: MessageRecord) {
        let dto = MessageDto::from(message.clone());
        let _ = self.app_handle.emit("new-message", &dto);

        // Native Windows notification when incoming message arrives
        if !message.is_outgoing {
            let _ = self
                .app_handle
                .notification()
                .builder()
                .title("EchoMesh")
                .body(&message.text)
                .show();
        }
    }

    fn on_message_status_updated(&self, message_id: String, status: DeliveryStatus) {
        let status_code = match status {
            DeliveryStatus::Sent => 1,
            DeliveryStatus::Relayed => 1,
            DeliveryStatus::Delivered => 1,
            DeliveryStatus::Failed => 2,
        };

        let _ = self.app_handle.emit(
            "message-status-updated",
            serde_json::json!({
                "id": message_id,
                "status": status_code,
            }),
        );
    }

    fn on_packet_received(&self, sender: Vec<u8>, data: Vec<u8>) {
        let _ = self.app_handle.emit(
            "packet-received",
            serde_json::json!({
                "sender_hex": hex::encode(sender),
                "data_len": data.len(),
            }),
        );
    }
}

/// Resolves the storage directory in %APPDATA%/EchoMesh on Windows.
pub fn get_app_data_path() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = PathBuf::from(appdata).join("EchoMesh");
            let _ = std::fs::create_dir_all(&path);
            return path;
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home).join(".config").join("EchoMesh");
        let _ = std::fs::create_dir_all(&path);
        return path;
    }

    let path = PathBuf::from("./data/EchoMesh");
    let _ = std::fs::create_dir_all(&path);
    path
}
