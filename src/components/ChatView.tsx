import React, { useState, useRef, useEffect } from "react";
import {
  Button,
  Textarea,
  Tooltip,
} from "@fluentui/react-components";
import {
  Send20Regular,
  ShieldCheckmark20Regular,
  Checkmark16Regular,
  CheckmarkCircle16Regular,
  Clock16Regular,
  ErrorCircle16Regular,
} from "@fluentui/react-icons";
import { MessageDto, ContactDto, ECHO_PEER_ID_HEX } from "../types";

interface ChatViewProps {
  currentContact?: ContactDto;
  messages: MessageDto[];
  onSendMessage: (text: string) => Promise<void>;
  isSending: boolean;
}

export const ChatView: React.FC<ChatViewProps> = ({
  currentContact,
  messages,
  onSendMessage,
  isSending,
}) => {
  const [inputText, setInputText] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = async () => {
    if (!inputText.trim() || isSending) return;
    const textToSend = inputText;
    setInputText("");
    await onSendMessage(textToSend);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // Quick Action Chips
  const sendPing = () => onSendMessage("PING");

  const sendNoisePayload = () => {
    const randomHex = Array.from({ length: 32 }, () =>
      Math.floor(Math.random() * 16).toString(16)
    ).join("");
    onSendMessage(`NOISE_PAYLOAD:${randomHex}`);
  };

  const send1420bTest = () => {
    // 1420b frame has 26b header, leaving exactly 1394 bytes maximum payload
    const prefix = "TEST_1420B_FRAME:";
    const fillLength = 1394 - prefix.length;
    const fill = "X".repeat(Math.max(0, fillLength));
    onSendMessage(prefix + fill);
  };

  const isEchoNode =
    currentContact?.is_echo_node ||
    currentContact?.peer_id.toLowerCase() === ECHO_PEER_ID_HEX.toLowerCase();

  const renderStatus = (msg: MessageDto, index: number) => {
    if (!msg.is_outgoing) return null;

    // Check if next incoming message was an echo reply to calculate RTT
    let rtt: number | undefined = msg.rtt_ms;
    if (rtt === undefined && index + 1 < messages.length) {
      const nextMsg = messages[index + 1];
      if (!nextMsg.is_outgoing && nextMsg.timestamp >= msg.timestamp) {
        rtt = Math.max(1, nextMsg.timestamp - msg.timestamp);
      }
    }

    if (rtt !== undefined && (msg.status === 1 || isEchoNode)) {
      return (
        <span className="rtt-tag">
          <CheckmarkCircle16Regular style={{ verticalAlign: "middle", marginRight: "3px" }} />
          Эхо получено ({rtt} мс)
        </span>
      );
    }

    switch (msg.status) {
      case 0:
        return (
          <Tooltip content="Отправка кадра в сокет..." relationship="description">
            <span style={{ display: "inline-flex", alignItems: "center", gap: "2px" }}>
              <Clock16Regular /> Отправляется
            </span>
          </Tooltip>
        );
      case 1:
        return (
          <Tooltip content="Кадр отправлен и подтвержден" relationship="description">
            <span style={{ display: "inline-flex", alignItems: "center", gap: "2px" }}>
              <Checkmark16Regular /> Отправлено
            </span>
          </Tooltip>
        );
      case 2:
      default:
        return (
          <Tooltip content="Ошибка отправки кадра" relationship="description">
            <span style={{ color: "#ff5252", display: "inline-flex", alignItems: "center", gap: "2px" }}>
              <ErrorCircle16Regular /> Ошибка
            </span>
          </Tooltip>
        );
    }
  };

  const formatTime = (timestamp: number) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  };

  return (
    <div className="chat-view">
      <div className="chat-header">
        <div className="chat-peer-info">
          <div className="chat-title-group">
            <h2>{currentContact?.name || "Echo Relay Node"}</h2>
            <p>
              0x{currentContact?.peer_id.slice(0, 8)}...{currentContact?.peer_id.slice(-8)}
            </p>
          </div>
        </div>

        <div className="chat-encryption-badge">
          <ShieldCheckmark20Regular style={{ color: "var(--accent-cyan)" }} />
          <span>ChaCha20-Poly1305 • 1420b frames</span>
        </div>
      </div>

      <div className="message-feed">
        {messages.length === 0 ? (
          <div style={{ textAlign: "center", color: "var(--text-secondary)", margin: "auto" }}>
            <p style={{ fontSize: "15px", marginBottom: "6px" }}>Нет сообщений</p>
            <p style={{ fontSize: "12px" }}>
              Используйте быстрые чипы ниже для проверки эхо-зеркалирования кадра.
            </p>
          </div>
        ) : (
          messages.map((msg, index) => (
            <div
              key={msg.id || index}
              className={`message-bubble-wrapper ${msg.is_outgoing ? "outgoing" : "incoming"}`}
            >
              <div className={`message-bubble ${msg.is_outgoing ? "outgoing" : "incoming"}`}>
                {msg.text}
              </div>
              <div className="message-meta">
                <span>{formatTime(msg.timestamp)}</span>
                {renderStatus(msg, index)}
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="chips-row">
        <Button size="small" appearance="subtle" onClick={sendPing} disabled={isSending}>
          [Ping]
        </Button>
        <Button size="small" appearance="subtle" onClick={sendNoisePayload} disabled={isSending}>
          [Noise Payload]
        </Button>
        <Button size="small" appearance="subtle" onClick={send1420bTest} disabled={isSending}>
          [1420b Test]
        </Button>
      </div>

      <div className="input-container">
        <Textarea
          className="input-box"
          value={inputText}
          onChange={(_, data) => setInputText(data.value)}
          onKeyDown={handleKeyDown}
          placeholder="Напишите сообщение... (Enter для отправки)"
          rows={1}
          style={{ resize: "none" }}
        />
        <Button
          appearance="primary"
          icon={<Send20Regular />}
          onClick={handleSend}
          disabled={!inputText.trim() || isSending}
        >
          Отправить
        </Button>
      </div>
    </div>
  );
};
