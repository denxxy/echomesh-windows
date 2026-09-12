import React from "react";
import { Button, Spinner } from "@fluentui/react-components";
import {
  ArrowSync20Regular,
  PersonAdd20Regular,
  Bot20Regular,
  Person20Regular,
} from "@fluentui/react-icons";
import { ContactDto, RelayStatusDto, ECHO_PEER_ID_HEX } from "../types";

interface SidebarProps {
  status: RelayStatusDto;
  contacts: ContactDto[];
  selectedPeerId: string;
  onSelectPeer: (peerId: string) => void;
  onReconnect: () => void;
  onOpenAddContact: () => void;
  isReconnecting: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  status,
  contacts,
  selectedPeerId,
  onSelectPeer,
  onReconnect,
  onOpenAddContact,
  isReconnecting,
}) => {
  const getStatusClass = (code: number) => {
    switch (code) {
      case 2:
      case 3:
        return "connected";
      case 1:
        return "connecting";
      default:
        return "offline";
    }
  };

  const getStatusText = (code: number) => {
    switch (code) {
      case 2:
        return `В сети (${status.ping_ms} мс)`;
      case 3:
        return "BLE Fallback";
      case 1:
        return "Подключение...";
      default:
        return "Отключен";
    }
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <div className="brand-row">
          <div className="brand-title">
            <div className="brand-logo-dot" />
            <span>EchoMesh</span>
          </div>
          <div className={`status-badge ${getStatusClass(status.connection_status)}`}>
            <div className="status-dot" />
            <span>{getStatusText(status.connection_status)}</span>
          </div>
        </div>

        <div className="relay-info-card">
          <div className="relay-endpoint">
            <span>{status.host}:{status.port}</span>
            <span>{status.connection_status === 2 ? "ONLINE" : "STANDBY"}</span>
          </div>
          <Button
            className="reconnect-btn"
            appearance="secondary"
            size="small"
            icon={isReconnecting ? <Spinner size="extra-tiny" /> : <ArrowSync20Regular />}
            onClick={onReconnect}
            disabled={isReconnecting}
          >
            {isReconnecting ? "Подключение..." : "Принудительное подключение"}
          </Button>
        </div>
      </div>

      <div className="contacts-section">
        <div className="section-label">Диалоги</div>

        {contacts.map((contact) => {
          const isActive = contact.peer_id.toLowerCase() === selectedPeerId.toLowerCase();
          const isEcho = contact.is_echo_node || contact.peer_id.toLowerCase() === ECHO_PEER_ID_HEX.toLowerCase();

          return (
            <div
              key={contact.peer_id}
              className={`contact-item ${isActive ? "active" : ""}`}
              onClick={() => onSelectPeer(contact.peer_id)}
            >
              <div className={`contact-avatar ${isEcho ? "echo" : ""}`}>
                {isEcho ? <Bot20Regular /> : <Person20Regular />}
              </div>
              <div className="contact-details">
                <div className="contact-name-row">
                  <span className="contact-name">{contact.name}</span>
                  {isEcho && <span className="echo-tag">LOOPBACK</span>}
                </div>
                <div className="contact-sub">
                  0x{contact.peer_id.slice(0, 6)}...{contact.peer_id.slice(-6)}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="sidebar-footer">
        <Button
          style={{ width: "100%" }}
          appearance="primary"
          icon={<PersonAdd20Regular />}
          onClick={onOpenAddContact}
        >
          Добавить контакт
        </Button>
      </div>
    </div>
  );
};
