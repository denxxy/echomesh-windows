import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "./components/Sidebar";
import { ChatView } from "./components/ChatView";
import { AddContactModal } from "./components/AddContactModal";
import {
  ContactDto,
  MessageDto,
  RelayStatusDto,
  ECHO_PEER_ID_HEX,
  DEFAULT_RELAY_HOST,
  DEFAULT_RELAY_PORT,
  DEFAULT_RELAY_PUBKEY_HEX,
  DEFAULT_SECRET_TOKEN_HEX,
} from "./types";

export const App: React.FC = () => {
  const [status, setStatus] = useState<RelayStatusDto>({
    connection_status: 0,
    status_text: "Offline",
    ping_ms: 0,
    host: DEFAULT_RELAY_HOST,
    port: DEFAULT_RELAY_PORT,
    public_key_hex: DEFAULT_RELAY_PUBKEY_HEX,
  });

  const [contacts, setContacts] = useState<ContactDto[]>([
    {
      peer_id: ECHO_PEER_ID_HEX,
      name: "Echo Relay Node",
      added_at: Date.now(),
      is_echo_node: true,
    },
  ]);

  const [selectedPeerId, setSelectedPeerId] = useState<string>(ECHO_PEER_ID_HEX);
  const [messages, setMessages] = useState<MessageDto[]>([]);
  const [isReconnecting, setIsReconnecting] = useState<boolean>(false);
  const [isSending, setIsSending] = useState<boolean>(false);
  const [isAddContactOpen, setIsAddContactOpen] = useState<boolean>(false);

  // Load history for selected peer
  const loadHistory = useCallback(async (peerId: string) => {
    try {
      const history = await invoke<MessageDto[]>("get_history", {
        peerIdHex: peerId,
        limit: 100,
      });
      setMessages(history);
    } catch (err) {
      console.error("Failed to load history:", err);
    }
  }, []);

  // Fetch status and contacts from Rust backend
  const refreshData = useCallback(async () => {
    try {
      const currentStatus = await invoke<RelayStatusDto>("get_status");
      setStatus(currentStatus);

      const contactList = await invoke<ContactDto[]>("get_contacts");
      if (contactList && contactList.length > 0) {
        setContacts(contactList);
      }
    } catch (err) {
      console.error("Failed to refresh status/contacts:", err);
    }
  }, []);

  // Connect to relay
  const connectToRelay = useCallback(async () => {
    setIsReconnecting(true);
    setStatus((prev) => ({ ...prev, connection_status: 1, status_text: "Connecting" }));
    try {
      await invoke("connect_to_relay", {
        host: DEFAULT_RELAY_HOST,
        port: DEFAULT_RELAY_PORT,
        publicKeyHex: DEFAULT_RELAY_PUBKEY_HEX,
        secretTokenHex: DEFAULT_SECRET_TOKEN_HEX,
      });
      await refreshData();
    } catch (err) {
      console.warn("Relay connect error (will stay standby or fallback):", err);
      await refreshData();
    } finally {
      setIsReconnecting(false);
    }
  }, [refreshData]);

  // Initial setup and event subscriptions
  useEffect(() => {
    let unlistenStatus: (() => void) | undefined;
    let unlistenMsg: (() => void) | undefined;
    let unlistenStatusUpdated: (() => void) | undefined;

    const setupListeners = async () => {
      // 1. Connection status change listener
      unlistenStatus = await listen<{ status: number; status_text: string }>(
        "connection-status-changed",
        (event) => {
          setStatus((prev) => ({
            ...prev,
            connection_status: event.payload.status,
            status_text: event.payload.status_text,
          }));
        }
      );

      // 2. Incoming new message listener
      unlistenMsg = await listen<MessageDto>("new-message", (event) => {
        const incoming = event.payload;
        setMessages((prev) => {
          // If already present, don't duplicate
          if (prev.some((m) => m.id === incoming.id)) return prev;
          return [...prev, incoming];
        });
      });

      // 3. Message status update listener (Sending -> Sent / Failed)
      unlistenStatusUpdated = await listen<{ id: string; status: number }>(
        "message-status-updated",
        (event) => {
          const { id, status: newStatus } = event.payload;
          setMessages((prev) =>
            prev.map((m) => (m.id === id ? { ...m, status: newStatus } : m))
          );
        }
      );

      // Refresh initial contacts and load echo node history
      await refreshData();
      await loadHistory(selectedPeerId);

      // Auto-connect to relay
      await connectToRelay();
    };

    setupListeners();

    return () => {
      if (unlistenStatus) unlistenStatus();
      if (unlistenMsg) unlistenMsg();
      if (unlistenStatusUpdated) unlistenStatusUpdated();
    };
  }, [connectToRelay, loadHistory, refreshData, selectedPeerId]);

  // Handle switching active conversation
  const handleSelectPeer = (peerId: string) => {
    setSelectedPeerId(peerId);
    loadHistory(peerId);
  };

  // Handle sending a new message
  const handleSendMessage = async (text: string) => {
    if (!text.trim()) return;
    setIsSending(true);

    const tempId = `temp_${Date.now()}`;
    const optimisticMsg: MessageDto = {
      id: tempId,
      conversation_peer_id: selectedPeerId,
      sender_peer_id: "0".repeat(64),
      text,
      timestamp: Date.now(),
      is_outgoing: true,
      status: 0, // Sending
    };

    setMessages((prev) => [...prev, optimisticMsg]);

    try {
      const sentMsg = await invoke<MessageDto>("send_message", {
        recipientIdHex: selectedPeerId,
        text,
      });

      setMessages((prev) =>
        prev.map((m) => (m.id === tempId ? sentMsg : m))
      );
    } catch (err) {
      console.error("Failed to send message:", err);
      setMessages((prev) =>
        prev.map((m) => (m.id === tempId ? { ...m, status: 2 } : m))
      );
    } finally {
      setIsSending(false);
    }
  };

  // Handle adding new contact
  const handleAddContact = async (peerIdHex: string, name: string) => {
    const newContact = await invoke<ContactDto>("add_contact", {
      peerIdHex,
      name,
    });
    setContacts((prev) => [...prev, newContact]);
    setSelectedPeerId(newContact.peer_id);
    setMessages([]);
  };

  const currentContact = contacts.find(
    (c) => c.peer_id.toLowerCase() === selectedPeerId.toLowerCase()
  ) || {
    peer_id: selectedPeerId,
    name: "Echo Relay Node",
    added_at: Date.now(),
    is_echo_node: true,
  };

  return (
    <div className="app-container">
      <Sidebar
        status={status}
        contacts={contacts}
        selectedPeerId={selectedPeerId}
        onSelectPeer={handleSelectPeer}
        onReconnect={connectToRelay}
        onOpenAddContact={() => setIsAddContactOpen(true)}
        isReconnecting={isReconnecting}
      />

      <ChatView
        currentContact={currentContact}
        messages={messages}
        onSendMessage={handleSendMessage}
        isSending={isSending}
      />

      <AddContactModal
        isOpen={isAddContactOpen}
        onClose={() => setIsAddContactOpen(false)}
        onAdd={handleAddContact}
      />
    </div>
  );
};

export default App;
