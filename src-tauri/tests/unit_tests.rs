use std::sync::atomic::Ordering;
use echomesh_windows_lib::state::{
    get_app_data_path, AppState, ContactDto, MessageDto, RelayStatusDto,
};
use echomesh_core::protocol::ECHO_SERVICE_PEER_ID;

#[test]
fn test_app_data_path_resolution() {
    let path = get_app_data_path();
    assert!(path.to_string_lossy().contains("EchoMesh"));
}

#[test]
fn test_app_state_initialization_and_status() {
    let temp_dir = std::env::temp_dir().join(format!("echomesh_win_test_{}", rand::random::<u32>()));
    let state = AppState::new(temp_dir);

    assert_eq!(state.connection_status.load(Ordering::SeqCst), 0);

    let code = state.set_status(echomesh_core::NetworkState::Connecting);
    assert_eq!(code, 1);
    assert_eq!(state.connection_status.load(Ordering::SeqCst), 1);

    let code2 = state.set_status(echomesh_core::NetworkState::ConnectedRealityRelay);
    assert_eq!(code2, 2);
    assert_eq!(state.connection_status.load(Ordering::SeqCst), 2);
}

#[test]
fn test_dto_serialization() {
    let msg = MessageDto {
        id: "msg_123".to_string(),
        conversation_peer_id: hex::encode(ECHO_SERVICE_PEER_ID),
        sender_peer_id: hex::encode([0u8; 32]),
        text: "PING".to_string(),
        timestamp: 1726000000000,
        is_outgoing: true,
        status: 1,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"text\":\"PING\""));
    assert!(json.contains(&hex::encode(ECHO_SERVICE_PEER_ID)));

    let deserialized: MessageDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "msg_123");
    assert_eq!(deserialized.text, "PING");
    assert!(deserialized.is_outgoing);
    assert_eq!(deserialized.status, 1);
}

#[test]
fn test_contact_dto_conversion() {
    let contact = echomesh_core::Contact {
        peer_id: ECHO_SERVICE_PEER_ID.to_vec(),
        name: "Echo Relay Node".to_string(),
        added_at: 1726000000000,
    };

    let dto = ContactDto::from(contact);
    assert_eq!(dto.peer_id, hex::encode(ECHO_SERVICE_PEER_ID));
    assert_eq!(dto.name, "Echo Relay Node");
    assert!(dto.is_echo_node);

    let user_peer_id = [0x42u8; 32];
    let user_contact = echomesh_core::Contact {
        peer_id: user_peer_id.to_vec(),
        name: "Alice".to_string(),
        added_at: 1726000000000,
    };
    let user_dto = ContactDto::from(user_contact);
    assert_eq!(user_dto.peer_id, hex::encode(user_peer_id));
    assert!(!user_dto.is_echo_node);
}

#[test]
fn test_relay_status_dto_json() {
    let status = RelayStatusDto {
        connection_status: 2,
        status_text: "Connected".to_string(),
        ping_ms: 38,
        host: "77.81.5.109".to_string(),
        port: 8443,
        public_key_hex: "a19be0e77828448ddf354673e55c0ae97cc523d28891db96bba81866c9acd606".to_string(),
    };

    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("77.81.5.109"));
    assert!(json.contains("\"connection_status\":2"));
    assert!(json.contains("\"ping_ms\":38"));
}
