/**
 * GENERATED FILE - DO NOT EDIT.
 * Source: protocol/schema/. Regenerate with `npm run generate` in protocol/.
 */

/**
 * payload.timestamp is diagnostic only. Online/offline decisions use the server monotonic clock (doc 11.1).
 */
export interface AgentHeartbeatMessage {
  protocolVersion: 1;
  type: 'agent.heartbeat';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * UTC RFC 3339 timestamp.
     */
    timestamp: string;
  };
}

export interface AgentHelloMessage {
  protocolVersion: 1;
  type: 'agent.hello';
  /**
   * Lowercase hyphenated UUID.
   */
  requestId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    agentVersion: string;
    platform: 'windows-x64' | 'ubuntu-x64';
    /**
     * @minItems 1
     * @maxItems 2
     */
    capabilities:
      ['terminal' | 'file-transfer'] | ['terminal' | 'file-transfer', 'terminal' | 'file-transfer'];
  };
}

export interface AgentHelloAckMessage {
  protocolVersion: 1;
  type: 'agent.helloAck';
  /**
   * Lowercase hyphenated UUID.
   */
  requestId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * UTC RFC 3339 timestamp.
     */
    serverTime: string;
    heartbeatIntervalMs: 20000;
  };
}

/**
 * Plugin sends it to end a session (reason=user). Agent sends it when the shell exits (reason=shell_exited, exitCode set) or the peer went away.
 */
export interface TerminalCloseMessage {
  protocolVersion: 1;
  type: 'terminal.close';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  sessionId: string;
  payload: {
    reason: 'user' | 'peer_disconnected' | 'shell_exited' | 'error';
    exitCode: number | null;
  };
}

/**
 * sessionId is null when the failure happens before a session exists (DEVICE_OFFLINE, DEVICE_BUSY, SHELL_START_FAILED in response to terminal.open); requestId echoes the failed request when there is one. message must be redacted per doc 8.8.6.
 */
export interface TerminalErrorMessage {
  protocolVersion: 1;
  type: 'terminal.error';
  requestId: string | null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: string | null;
  payload: {
    code:
      | 'AUTH_EXPIRED'
      | 'AUTH_INVALID'
      | 'PAIRING_CODE_INVALID'
      | 'DEVICE_FORBIDDEN'
      | 'DEVICE_OFFLINE'
      | 'DEVICE_BUSY'
      | 'SHELL_START_FAILED'
      | 'RELAY_DISCONNECTED'
      | 'INVALID_DROP'
      | 'INVALID_PATH'
      | 'WRITE_FAILED'
      | 'TRANSFER_FAILED'
      | 'QUOTA_EXCEEDED'
      | 'RATE_LIMITED'
      | 'BACKPRESSURE_LIMIT'
      | 'SESSION_TIMEOUT'
      | 'PROTOCOL_ERROR'
      | 'INTERNAL_ERROR';
    message: string;
  };
}

export interface TerminalOpenMessage {
  protocolVersion: 1;
  type: 'terminal.open';
  /**
   * Lowercase hyphenated UUID.
   */
  requestId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    cols: number;
    rows: number;
  };
}

/**
 * Direct response to terminal.open: requestId matches, and the envelope carries the sessionId minted by the Agent (doc 8.8.3).
 */
export interface TerminalOpenedMessage {
  protocolVersion: 1;
  type: 'terminal.opened';
  /**
   * Lowercase hyphenated UUID.
   */
  requestId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  sessionId: string;
  payload: {
    shell: string;
  };
}

export interface TerminalResizeMessage {
  protocolVersion: 1;
  type: 'terminal.resize';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  sessionId: string;
  payload: {
    cols: number;
    rows: number;
  };
}

/**
 * Shell integration events produced by the Agent's port of rust-servers/src/pty/osc_scanner.rs. The payload is shape-identical to Termy's local ShellEvent so the plugin can forward it to the existing handler unchanged. cwd is deliberately absent: the plugin derives it by parsing the xterm buffer (extractCwdFromPromptLines), and the terminal output bytes that parsing needs are forwarded verbatim, so cwd tracking already works in remote mode without protocol support.
 */
export interface TerminalShellEventMessage {
  protocolVersion: 1;
  type: 'terminal.shellEvent';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  sessionId: string;
  payload: {
    /**
     * Mirrors Termy's local ShellEventType exactly, so the plugin can hand payload straight to its existing shell-event handler.
     */
    type: 'prompt_start' | 'command_start' | 'command_executed' | 'command_end';
    /**
     * Which shell-integration sequence produced the event, mirroring Termy's local ShellEventSource.
     */
    source: 'osc133' | 'osc633';
    exitCode: number | null;
  };
}

/**
 * Plugin-side abort: read failure, user cancel, or leaving remote mode. The Agent stops accepting frames for this transferId, closes the open file handle and replies transfer.result with success=false. Partial files may remain.
 */
export interface TransferAbortMessage {
  protocolVersion: 1;
  type: 'transfer.abort';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
    code:
      | 'AUTH_EXPIRED'
      | 'AUTH_INVALID'
      | 'PAIRING_CODE_INVALID'
      | 'DEVICE_FORBIDDEN'
      | 'DEVICE_OFFLINE'
      | 'DEVICE_BUSY'
      | 'SHELL_START_FAILED'
      | 'RELAY_DISCONNECTED'
      | 'INVALID_DROP'
      | 'INVALID_PATH'
      | 'WRITE_FAILED'
      | 'TRANSFER_FAILED'
      | 'QUOTA_EXCEEDED'
      | 'RATE_LIMITED'
      | 'BACKPRESSURE_LIMIT'
      | 'SESSION_TIMEOUT'
      | 'PROTOCOL_ERROR'
      | 'INTERNAL_ERROR';
  };
}

/**
 * Direct response to transfer.start. grantedBytes is the initial credit window (doc 8.6): the plugin may not send file frames before this message, and never more than grantedBytes cumulative bytes.
 */
export interface TransferAcceptedMessage {
  protocolVersion: 1;
  type: 'transfer.accepted';
  /**
   * Lowercase hyphenated UUID.
   */
  requestId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
    grantedBytes: number;
  };
}

export interface TransferCompleteMessage {
  protocolVersion: 1;
  type: 'transfer.complete';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
  };
}

/**
 * Credit top-up, sent after every 1 MiB flushed to disk. grantedBytes is the CUMULATIVE authorisation for the whole transfer and must increase monotonically; the receiver keeps the maximum it has seen. Upper bound is the 256 MiB per-transfer cap.
 */
export interface TransferCreditMessage {
  protocolVersion: 1;
  type: 'transfer.credit';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
    grantedBytes: number;
  };
}

/**
 * Marks the end of one file. sentSize is authoritative for the success check (doc 10.4); a zero value means an empty file, for which no chunk frame is sent and the Agent must still create and close a 0-byte file.
 */
export interface TransferFileEndMessage {
  protocolVersion: 1;
  type: 'transfer.fileEnd';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
    fileIndex: number;
    /**
     * Per-file byte limit, 64 MiB (doc 4.12 / 8.4).
     */
    sentSize: number;
  };
}

/**
 * Terminal outcome of one transfer. Not a direct response to transfer.start (that is transfer.accepted), so requestId is null and correlation is by transferId. code is null exactly when success is true - a cross-field rule enforced in code.
 */
export interface TransferResultMessage {
  protocolVersion: 1;
  type: 'transfer.result';
  requestId: null;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
    success: boolean;
    code:
      | (
          | 'AUTH_EXPIRED'
          | 'AUTH_INVALID'
          | 'PAIRING_CODE_INVALID'
          | 'DEVICE_FORBIDDEN'
          | 'DEVICE_OFFLINE'
          | 'DEVICE_BUSY'
          | 'SHELL_START_FAILED'
          | 'RELAY_DISCONNECTED'
          | 'INVALID_DROP'
          | 'INVALID_PATH'
          | 'WRITE_FAILED'
          | 'TRANSFER_FAILED'
          | 'QUOTA_EXCEEDED'
          | 'RATE_LIMITED'
          | 'BACKPRESSURE_LIMIT'
          | 'SESSION_TIMEOUT'
          | 'PROTOCOL_ERROR'
          | 'INTERNAL_ERROR'
        )
      | null;
    message: string;
  };
}

/**
 * File transfer is independent of any terminal session, so sessionId is null. entries[].index must run 0..n-1 with no gaps and rootNote must equal entries[0].relativePath; both are cross-field rules enforced in code, not expressible here.
 */
export interface TransferStartMessage {
  protocolVersion: 1;
  type: 'transfer.start';
  /**
   * Lowercase hyphenated UUID.
   */
  requestId: string;
  /**
   * Lowercase hyphenated UUID.
   */
  deviceId: string;
  sessionId: null;
  payload: {
    /**
     * Lowercase hyphenated UUID.
     */
    transferId: string;
    /**
     * Vault-relative path. Always '/'-separated. Structural safety (no '..', no drive letter, no UNC root, no absolute form, no empty segment, no NUL) is enforced in code per doc 10.3; maxLength here counts UTF-16 code units and is an upper bound only, the 1024-byte UTF-8 limit is enforced in code.
     */
    rootNote: string;
    /**
     * @minItems 1
     * @maxItems 256
     */
    entries: [FileEntry, ...FileEntry[]];
  };
}
export interface FileEntry {
  index: number;
  /**
   * Vault-relative path. Always '/'-separated. Structural safety (no '..', no drive letter, no UNC root, no absolute form, no empty segment, no NUL) is enforced in code per doc 10.3; maxLength here counts UTF-16 code units and is an upper bound only, the 1024-byte UTF-8 limit is enforced in code.
   */
  relativePath: string;
  /**
   * Per-file byte limit, 64 MiB (doc 4.12 / 8.4).
   */
  size: number;
}

export type ControlMessage =
    | AgentHeartbeatMessage
    | AgentHelloMessage
    | AgentHelloAckMessage
    | TerminalCloseMessage
    | TerminalErrorMessage
    | TerminalOpenMessage
    | TerminalOpenedMessage
    | TerminalResizeMessage
    | TerminalShellEventMessage
    | TransferAbortMessage
    | TransferAcceptedMessage
    | TransferCompleteMessage
    | TransferCreditMessage
    | TransferFileEndMessage
    | TransferResultMessage
    | TransferStartMessage;

export interface ControlMessageByType {
    'agent.heartbeat': AgentHeartbeatMessage;
    'agent.hello': AgentHelloMessage;
    'agent.helloAck': AgentHelloAckMessage;
    'terminal.close': TerminalCloseMessage;
    'terminal.error': TerminalErrorMessage;
    'terminal.open': TerminalOpenMessage;
    'terminal.opened': TerminalOpenedMessage;
    'terminal.resize': TerminalResizeMessage;
    'terminal.shellEvent': TerminalShellEventMessage;
    'transfer.abort': TransferAbortMessage;
    'transfer.accepted': TransferAcceptedMessage;
    'transfer.complete': TransferCompleteMessage;
    'transfer.credit': TransferCreditMessage;
    'transfer.fileEnd': TransferFileEndMessage;
    'transfer.result': TransferResultMessage;
    'transfer.start': TransferStartMessage;
}

export const CONTROL_MESSAGE_TYPES = [
    'agent.heartbeat',
    'agent.hello',
    'agent.helloAck',
    'terminal.close',
    'terminal.error',
    'terminal.open',
    'terminal.opened',
    'terminal.resize',
    'terminal.shellEvent',
    'transfer.abort',
    'transfer.accepted',
    'transfer.complete',
    'transfer.credit',
    'transfer.fileEnd',
    'transfer.result',
    'transfer.start',
] as const;

export type ControlMessageType = (typeof CONTROL_MESSAGE_TYPES)[number];
