export interface MessageDto {
  id: string;
  conversation_peer_id: string; // 64 hex chars
  sender_peer_id: string;       // 64 hex chars
  text: string;
  timestamp: number;            // milliseconds
  is_outgoing: bool;
  status: number;               // 0 = Sending, 1 = Sent, 2 = Failed
  rtt_ms?: number;              // optional calculated round-trip time
}

export type bool = boolean;

export interface ContactDto {
  peer_id: string;              // 64 hex chars
  name: string;
  added_at: number;
  is_echo_node: boolean;
}

export interface RelayStatusDto {
  connection_status: number;    // 0: Offline, 1: Connecting, 2: Connected, 3: BLE Fallback
  status_text: string;
  ping_ms: number;
  host: string;
  port: number;
  public_key_hex: string;
}

export const ECHO_PEER_ID_HEX = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
export const DEFAULT_RELAY_HOST = "77.81.5.109";
export const DEFAULT_RELAY_PORT = 8443;
export const DEFAULT_RELAY_PUBKEY_HEX = "a19be0e77828448ddf354673e55c0ae97cc523d28891db96bba81866c9acd606";
export const DEFAULT_SECRET_TOKEN_HEX = "651380e1cb3464e95878c6d6aebca5af3b0686895912f925365d1988a1d6a102";
