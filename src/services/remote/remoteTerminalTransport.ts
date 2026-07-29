import type {
  ControlMessage,
  TerminalCloseMessage,
  TerminalErrorMessage,
  TerminalOpenMessage,
  TerminalOpenedMessage,
  TerminalResizeMessage,
} from '../../protocol/generated/messages.ts';
import {
  decodeFrame,
  encodeFrame,
  KIND_TERMINAL_INPUT,
  KIND_TERMINAL_OUTPUT,
  TERMINAL_FILE_INDEX,
  TERMINAL_PAYLOAD_MAX,
} from './frameCodec.ts';
import type { RelayControlConnection } from './relayClient.ts';
import {
  DisposableBag,
  toDisposable,
  type Disposable,
  type ShellEventPayload,
  type TerminalExitEvent,
  type TerminalOpenOptions,
  type TerminalSessionInfo,
  type TerminalTransport,
} from './transport.ts';

const OPEN_TIMEOUT_MS = 15_000;

export interface RemoteTerminalDependencies {
  createRequestId: () => string;
  setTimeout: (handler: () => void, timeoutMs: number) => number;
  clearTimeout: (timer: number) => void;
  openTimeoutMs: number;
}

const defaultDependencies: RemoteTerminalDependencies = {
  createRequestId: () => crypto.randomUUID(),
  setTimeout: (handler, timeoutMs) => window.setTimeout(handler, timeoutMs),
  clearTimeout: (timer) => window.clearTimeout(timer),
  openTimeoutMs: OPEN_TIMEOUT_MS,
};

interface PendingOpen {
  requestId: string;
  resolve: (info: TerminalSessionInfo) => void;
  reject: (error: Error) => void;
  timer: number;
}

export class RemoteTerminalTransport implements TerminalTransport {
  private readonly connection: RelayControlConnection;
  private readonly deviceId: string;
  private readonly dependencies: RemoteTerminalDependencies;
  private readonly subscriptions = new DisposableBag();
  private readonly dataHandlers = new Set<(data: Uint8Array) => void>();
  private readonly exitHandlers = new Set<(event: TerminalExitEvent) => void>();
  private readonly errorHandlers = new Set<(code: string, message: string) => void>();
  private readonly shellEventHandlers = new Set<(event: ShellEventPayload) => void>();
  private sessionId: string | null = null;
  private nextInputOffset = 0;
  private pendingOpen: PendingOpen | null = null;

  constructor(
    connection: RelayControlConnection,
    deviceId: string,
    dependencies: Partial<RemoteTerminalDependencies> = {}
  ) {
    this.connection = connection;
    this.deviceId = deviceId;
    this.dependencies = { ...defaultDependencies, ...dependencies };
    this.subscriptions.add(connection.onControlMessage((message) => this.handleControl(message)));
    this.subscriptions.add(connection.onBinary((data) => this.handleBinary(data)));
    this.subscriptions.add(connection.onClose(() => this.handleDisconnect('Relay connection closed')));
    this.subscriptions.add(connection.onError((error) => this.handleDisconnect(error.message)));
  }

  open(options: TerminalOpenOptions): Promise<TerminalSessionInfo> {
    if (this.pendingOpen !== null || this.sessionId !== null) {
      return Promise.reject(new Error('Remote terminal is already open'));
    }

    const requestId = this.dependencies.createRequestId();
    const message: TerminalOpenMessage = {
      protocolVersion: 1,
      type: 'terminal.open',
      requestId,
      deviceId: this.deviceId,
      sessionId: null,
      payload: { cols: options.cols, rows: options.rows },
    };

    return new Promise((resolve, reject) => {
      const timer = this.dependencies.setTimeout(() => {
        if (this.pendingOpen?.requestId !== requestId) return;
        this.pendingOpen = null;
        reject(new Error('Remote terminal open timed out after 15 seconds'));
      }, this.dependencies.openTimeoutMs);
      this.pendingOpen = { requestId, resolve, reject, timer };
      try {
        this.connection.sendJson(message);
      } catch (error) {
        this.clearPendingOpen();
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  write(data: Uint8Array): void {
    const sessionId = this.requireSessionId();
    for (let start = 0; start < data.length; start += TERMINAL_PAYLOAD_MAX) {
      const payload = data.subarray(start, start + TERMINAL_PAYLOAD_MAX);
      this.connection.sendBinary(encodeFrame({
        kind: KIND_TERMINAL_INPUT,
        streamId: sessionId,
        fileIndex: TERMINAL_FILE_INDEX,
        offset: this.nextInputOffset,
        payload,
      }));
      this.nextInputOffset += payload.length;
    }
  }

  resize(cols: number, rows: number): void {
    const message: TerminalResizeMessage = {
      protocolVersion: 1,
      type: 'terminal.resize',
      requestId: null,
      deviceId: this.deviceId,
      sessionId: this.requireSessionId(),
      payload: { cols, rows },
    };
    this.connection.sendJson(message);
  }

  close(): Promise<void> {
    const sessionId = this.sessionId;
    if (sessionId !== null) {
      const message: TerminalCloseMessage = {
        protocolVersion: 1,
        type: 'terminal.close',
        requestId: null,
        deviceId: this.deviceId,
        sessionId,
        payload: { reason: 'user', exitCode: null },
      };
      this.connection.sendJson(message);
      this.sessionId = null;
    }
    this.rejectPendingOpen(new Error('Remote terminal closed'));
    this.subscriptions.dispose();
    return Promise.resolve();
  }

  onData(handler: (data: Uint8Array) => void): Disposable {
    return subscribe(this.dataHandlers, handler);
  }

  onExit(handler: (event: TerminalExitEvent) => void): Disposable {
    return subscribe(this.exitHandlers, handler);
  }

  onError(handler: (code: string, message: string) => void): Disposable {
    return subscribe(this.errorHandlers, handler);
  }

  onShellEvent(handler: (event: ShellEventPayload) => void): Disposable {
    return subscribe(this.shellEventHandlers, handler);
  }

  private handleControl(message: ControlMessage): void {
    if (message.deviceId !== this.deviceId) return;

    switch (message.type) {
      case 'terminal.opened':
        this.handleOpened(message);
        break;
      case 'terminal.close':
        this.handleTerminalClose(message);
        break;
      case 'terminal.error':
        this.handleTerminalError(message);
        break;
      case 'terminal.shellEvent':
        if (message.sessionId === this.sessionId) {
          for (const handler of this.shellEventHandlers) handler(message.payload);
        }
        break;
    }
  }

  private handleOpened(message: TerminalOpenedMessage): void {
    if (this.pendingOpen?.requestId !== message.requestId) return;
    const pending = this.clearPendingOpen();
    this.sessionId = message.sessionId;
    this.nextInputOffset = 0;
    pending?.resolve({ sessionId: message.sessionId, shell: message.payload.shell });
  }

  private handleTerminalClose(message: TerminalCloseMessage): void {
    if (message.sessionId !== this.sessionId) return;
    this.sessionId = null;
    for (const handler of this.exitHandlers) handler(message.payload);
  }

  private handleTerminalError(message: TerminalErrorMessage): void {
    if (message.requestId !== null && message.requestId === this.pendingOpen?.requestId) {
      this.rejectPendingOpen(new Error(`${message.payload.code}: ${message.payload.message}`));
    }
    if (message.sessionId === null || message.sessionId === this.sessionId) {
      for (const handler of this.errorHandlers) {
        handler(message.payload.code, message.payload.message);
      }
    }
  }

  private handleBinary(data: Uint8Array): void {
    try {
      const frame = decodeFrame(data);
      if (frame.kind !== KIND_TERMINAL_OUTPUT || frame.streamId !== this.sessionId) return;
      for (const handler of this.dataHandlers) handler(frame.payload);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      for (const handler of this.errorHandlers) handler('PROTOCOL_ERROR', message);
    }
  }

  private handleDisconnect(message: string): void {
    this.rejectPendingOpen(new Error(message));
    if (this.sessionId !== null) {
      this.sessionId = null;
      for (const handler of this.errorHandlers) handler('RELAY_DISCONNECTED', message);
    }
  }

  private requireSessionId(): string {
    if (this.sessionId === null) throw new Error('Remote terminal is not open');
    return this.sessionId;
  }

  private clearPendingOpen(): PendingOpen | null {
    const pending = this.pendingOpen;
    if (pending !== null) this.dependencies.clearTimeout(pending.timer);
    this.pendingOpen = null;
    return pending;
  }

  private rejectPendingOpen(error: Error): void {
    this.clearPendingOpen()?.reject(error);
  }
}

function subscribe<T>(handlers: Set<T>, handler: T): Disposable {
  handlers.add(handler);
  return toDisposable(() => handlers.delete(handler));
}