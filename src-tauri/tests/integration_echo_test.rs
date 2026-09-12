use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use echomesh_core::protocol::ECHO_SERVICE_PEER_ID;
use echomesh_core::{
    CoreEventsListener, DeliveryStatus, EchoMeshClient, MessageRecord, NetworkState,
};

struct MockEventListener {
    echo_received: AtomicBool,
    received_text: std::sync::Mutex<Option<String>>,
}

impl MockEventListener {
    fn new() -> Self {
        Self {
            echo_received: AtomicBool::new(false),
            received_text: std::sync::Mutex::new(None),
        }
    }
}

impl CoreEventsListener for MockEventListener {
    fn on_state_changed(&self, _state: NetworkState) {}

    fn on_message_received(&self, message: MessageRecord) {
        if !message.is_outgoing {
            *self.received_text.lock().unwrap() = Some(message.text);
            self.echo_received.store(true, Ordering::SeqCst);
        }
    }

    fn on_message_status_updated(&self, _message_id: String, _status: DeliveryStatus) {}

    fn on_packet_received(&self, _sender: Vec<u8>, _data: Vec<u8>) {}
}

#[test]
fn test_echo_loopback_and_sqlite_history_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("echomesh_e2e_win_{}", rand::random::<u32>()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let listener = Arc::new(MockEventListener::new());

    struct Bridge(Arc<MockEventListener>);
    impl CoreEventsListener for Bridge {
        fn on_state_changed(&self, s: NetworkState) { self.0.on_state_changed(s); }
        fn on_message_received(&self, m: MessageRecord) { self.0.on_message_received(m); }
        fn on_message_status_updated(&self, id: String, st: DeliveryStatus) { self.0.on_message_status_updated(id, st); }
        fn on_packet_received(&self, sender: Vec<u8>, data: Vec<u8>) { self.0.on_packet_received(sender, data); }
    }

    let client = EchoMeshClient::new(
        temp_dir.to_string_lossy().to_string(),
        Box::new(Bridge(listener.clone())),
    )
    .expect("EchoMeshClient must initialize with SQLite storage");

    // 1. Verify default seed contact: Echo Relay Node
    let contacts = client.get_contacts().expect("get_contacts must succeed");
    assert!(
        contacts.iter().any(|c| c.peer_id == ECHO_SERVICE_PEER_ID),
        "Default Echo Relay Node ([0xEE; 32]) must be seeded"
    );

    // 2. Send ping message to Echo Node
    let echo_hex = hex::encode(ECHO_SERVICE_PEER_ID);
    let sent_record = client
        .send_chat_message(echo_hex.clone(), "PING".to_string())
        .expect("send_chat_message must succeed");

    assert_eq!(sent_record.text, "PING");
    assert!(sent_record.is_outgoing);
    assert_eq!(sent_record.status, 1);

    // 3. Wait for echo loopback event
    let start = std::time::Instant::now();
    while !listener.echo_received.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(25));
    }

    assert!(
        listener.echo_received.load(Ordering::SeqCst),
        "Echo response must be received by listener"
    );

    let received_text = listener.received_text.lock().unwrap().clone();
    assert!(
        received_text.is_some() && received_text.unwrap().contains("PING"),
        "Echo response text must contain PING"
    );

    // 4. Verify message history persistence in SQLite
    let messages = client
        .get_messages(echo_hex, 10)
        .expect("get_messages must succeed");

    assert!(messages.len() >= 2, "History must contain both outgoing and incoming echo messages");
    assert_eq!(messages[0].text, "PING");
    assert!(messages[1].text.contains("PING"));

    let _ = client.shutdown();
}
