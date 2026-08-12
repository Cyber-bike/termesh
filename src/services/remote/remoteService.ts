import type { ControlMessage, TransferStartMessage } from '../../protocol/generated/messages.ts';
import { checkQuotas, collect, type CollectedFile, type LinkSource } from './noteCollector.ts';
import {
  RelayClient,
  type Device,
  type PairingCodeCreated,
  type RelayAuthSession,
  type RelayControlConnection,
} from './relayClient.ts';
import { RemoteTerminalTransport } from './remoteTerminalTransport.ts';
import { TransferSender, type TransferOutcome } from './transferSender.ts';
import type { Disposable, TerminalTransport } from './transport.ts';

const DEVICE_POLL_INTERVAL_MS = 15_000;
const TRANSFER_ACCEPT_TIMEOUT_MS = 15_000;

export interface RemoteConnectionConfig {
  relayUrl: string;
  deviceId: string | null;
  authSession: PersistedRemoteAuthSession | null;
}

export interface PersistedRemoteAuthSession extends RelayAuthSession {
  login: string;
}

export interface RemoteServiceSnapshot {
  authenticated: boolean;
  connected: boolean;
  devices: Device[];
  error: string | null;
}

interface PendingTransferStart {
  requestId: string;
  resolve: (grantedBytes: number) => void;
  reject: (error: Error) => void;
  timer: number;
}

export interface RemoteServiceDependencies {
  createClient: (relayUrl: string) => RelayClient;
  createId: () => string;
  setInterval: (handler: () => void, timeoutMs: number) => number;
  clearInterval: (timer: number) => void;
  setTimeout: (handler: () => void, timeoutMs: number) => number;
  clearTimeout: (timer: number) => void;
  now: () => number;
  saveAuthSession: (session: PersistedRemoteAuthSession | null) => void;
}

const defaultDependencies: RemoteServiceDependencies = {
  createClient: (relayUrl) => new RelayClient(relayUrl),
  createId: () => crypto.randomUUID(),
  setInterval: (handler, timeoutMs) => window.setInterval(handler, timeoutMs),
  clearInterval: (timer) => window.clearInterval(timer),
  setTimeout: (handler, timeoutMs) => window.setTimeout(handler, timeoutMs),
  clearTimeout: (timer) => window.clearTimeout(timer),
  now: Date.now,
  saveAuthSession: () => undefined,
};

export class RemoteService {
  private readonly getConfig: () => RemoteConnectionConfig;
  private readonly isOffline: () => boolean;
  private readonly dependencies: RemoteServiceDependencies;
  private readonly listeners = new Set<(snapshot: RemoteServiceSnapshot) => void>();
  private readonly senders = new Map<string, TransferSender>();
  private readonly pendingTransfers = new Map<string, PendingTransferStart>();
  private client: RelayClient | null = null;
  private clientUrl: string | null = null;
  private connection: RelayControlConnection | null = null;
  private connectionSubscriptions: Disposable[] = [];
  private pollTimer: number | null = null;
  private remoteMode = false;
  private snapshot: RemoteServiceSnapshot = {
    authenticated: false,
    connected: false,
    devices: [],
    error: null,
  };

  constructor(
    getConfig: () => RemoteConnectionConfig,
    isOffline: () => boolean,
    dependencies: Partial<RemoteServiceDependencies> = {},
  ) {
    this.getConfig = getConfig;
    this.isOffline = isOffline;
    this.dependencies = { ...defaultDependencies, ...dependencies };
  }

  getSnapshot(): RemoteServiceSnapshot {
    return { ...this.snapshot, devices: [...this.snapshot.devices] };
  }

  onDidChange(listener: (snapshot: RemoteServiceSnapshot) => void): Disposable {
    this.listeners.add(listener);
    return { dispose: () => this.listeners.delete(listener) };
  }

  async login(login: string, password: string): Promise<void> {
    this.assertOnline();
    const client = this.getClient(true);
    const response = await client.login(login, password);
    await this.finishAuthentication(response);
  }

  async register(login: string, password: string): Promise<void> {
    this.assertOnline();
    const client = this.getClient(true);
    const response = await client.register(login, password);
    await this.finishAuthentication(response);
  }

  private async finishAuthentication(response: {
    accessToken: string;
    expiresIn: number;
    user: { login: string };
  }): Promise<void> {
    this.dependencies.saveAuthSession({
      accessToken: response.accessToken,
      expiresAt: this.dependencies.now() + response.expiresIn * 1000,
      login: response.user.login,
    });
    this.updateSnapshot({ authenticated: true, error: null });
    await this.refreshDevices();
  }

  async restoreAuthentication(): Promise<boolean> {
    const session = this.getConfig().authSession;
    if (!session || this.isOffline()) return false;
    try {
      const client = this.getClient();
      client.restoreAuthentication(session);
      this.updateSnapshot({ authenticated: true, error: null });
      await this.refreshDevices();
      return true;
    } catch {
      this.dependencies.saveAuthSession(null);
      this.client = null;
      this.clientUrl = null;
      this.updateSnapshot({ authenticated: false, devices: [], error: null });
      return false;
    }
  }

  logout(): void {
    this.disconnect();
    this.client = null;
    this.clientUrl = null;
    this.dependencies.saveAuthSession(null);
    this.updateSnapshot({ authenticated: false, devices: [], error: null });
  }

  async createPairingCode(): Promise<PairingCodeCreated> {
    this.assertOnline();
    return this.requireClient().createPairingCode();
  }

  async revokePairingCode(id: string): Promise<void> {
    this.assertOnline();
    await this.requireClient().revokePairingCode(id);
  }

  async refreshDevices(): Promise<Device[]> {
    this.assertOnline();
    try {
      const response = await this.requireClient().listDevices();
      this.updateSnapshot({ devices: response.devices, error: null });
      return response.devices;
    } catch (error) {
      this.handleRequestError(error);
      throw error;
    }
  }

  async deleteDevice(id: string): Promise<void> {
    this.assertOnline();
    await this.requireClient().deleteDevice(id);
    await this.refreshDevices();
  }

  async connect(): Promise<void> {
    this.assertOnline();
    if (this.connection) return;
    const connection = await this.requireClient().connectControl();
    this.connection = connection;
    this.connectionSubscriptions = [
      connection.onControlMessage((message) => this.handleControlMessage(message)),
      connection.onClose(() => this.handleConnectionLost('Relay connection closed')),
      connection.onError((error) => this.handleConnectionLost(error.message)),
    ];
    this.stopPolling();
    this.updateSnapshot({ connected: true, error: null });
  }

  disconnect(): void {
    this.finishConnection('Remote connection closed');
    this.updateSnapshot({ connected: false });
    this.updatePolling();
  }

  setRemoteMode(enabled: boolean): void {
    this.remoteMode = enabled;
    if (!enabled) this.disconnect();
    this.updatePolling();
  }

  updateConfiguration(): void {
    if (this.isOffline()) {
      this.disconnect();
      return;
    }
    if (this.clientUrl !== null && this.clientUrl !== this.getConfig().relayUrl) {
      this.logout();
    }
    this.updatePolling();
  }

  createTerminalTransport(): TerminalTransport {
    const deviceId = this.getConfig().deviceId;
    if (!this.connection || !deviceId) {
      throw new Error('Select a device and connect before opening a remote terminal');
    }
    return new RemoteTerminalTransport(this.connection, deviceId);
  }

  async transfer(
    source: LinkSource,
    readFile: (path: string) => Promise<Uint8Array>,
  ): Promise<TransferOutcome> {
    this.assertOnline();
    const connection = this.connection;
    const deviceId = this.getConfig().deviceId;
    if (!connection || !deviceId) throw new Error('Remote terminal is not connected');

    const collected = collect(source);
    if (!collected.ok) throw new Error(collected.error ?? 'Unable to collect dropped note');
    const quota = checkQuotas(collected.files);
    if (!quota.ok) throw new Error(quota.error ?? 'Transfer quota exceeded');

    const transferId = this.dependencies.createId();
    const requestId = this.dependencies.createId();
    const initialCredit = await this.startTransfer(connection, deviceId, transferId, requestId, collected.files);
    const sender = new TransferSender(connection, deviceId, transferId, collected.files, readFile, initialCredit);
    this.senders.set(transferId, sender);
    try {
      return await sender.run();
    } finally {
      this.senders.delete(transferId);
    }
  }

  dispose(): void {
    this.remoteMode = false;
    this.stopPolling();
    this.finishConnection('Remote service disposed');
    this.listeners.clear();
    this.client = null;
  }

  private startTransfer(
    connection: RelayControlConnection,
    deviceId: string,
    transferId: string,
    requestId: string,
    files: CollectedFile[],
  ): Promise<number> {
    const message: TransferStartMessage = {
      protocolVersion: 1,
      type: 'transfer.start',
      requestId,
      deviceId,
      sessionId: null,
      payload: {
        transferId,
        rootNote: files[0].relativePath,
        entries: files.map((file) => ({ ...file })) as TransferStartMessage['payload']['entries'],
      },
    };

    return new Promise((resolve, reject) => {
      const timer = this.dependencies.setTimeout(() => {
        this.pendingTransfers.delete(transferId);
        reject(new Error('Remote transfer was not accepted within 15 seconds'));
      }, TRANSFER_ACCEPT_TIMEOUT_MS);
      this.pendingTransfers.set(transferId, { requestId, resolve, reject, timer });
      try {
        connection.sendJson(message);
      } catch (error) {
        this.rejectPendingTransfer(transferId, error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  private handleControlMessage(message: ControlMessage): void {
    if (message.type === 'transfer.accepted') {
      const pending = this.pendingTransfers.get(message.payload.transferId);
      if (pending?.requestId !== message.requestId) return;
      this.dependencies.clearTimeout(pending.timer);
      this.pendingTransfers.delete(message.payload.transferId);
      pending.resolve(message.payload.grantedBytes);
      return;
    }
    if (message.type === 'transfer.credit') {
      this.senders.get(message.payload.transferId)?.grantCredit(message.payload.grantedBytes);
      return;
    }
    if (message.type === 'transfer.result') {
      const pending = this.pendingTransfers.get(message.payload.transferId);
      if (pending) {
        this.rejectPendingTransfer(message.payload.transferId, new Error(message.payload.message));
      }
      this.senders.get(message.payload.transferId)?.complete({
        success: message.payload.success,
        code: message.payload.code,
        message: message.payload.message,
        destinationPath: message.payload.destinationPath,
      });
    }
  }

  private handleConnectionLost(message: string): void {
    this.finishConnection(message);
    this.updateSnapshot({ connected: false, error: message });
    this.updatePolling();
  }

  private finishConnection(message: string): void {
    for (const subscription of this.connectionSubscriptions.splice(0)) subscription.dispose();
    const connection = this.connection;
    this.connection = null;
    for (const transferId of [...this.pendingTransfers.keys()]) {
      this.rejectPendingTransfer(transferId, new Error(message));
    }
    for (const sender of this.senders.values()) {
      try { sender.abandon(message); } catch { /* the connection is already unavailable */ }
    }
    this.senders.clear();
    connection?.dispose();
  }

  private rejectPendingTransfer(transferId: string, error: Error): void {
    const pending = this.pendingTransfers.get(transferId);
    if (!pending) return;
    this.dependencies.clearTimeout(pending.timer);
    this.pendingTransfers.delete(transferId);
    pending.reject(error);
  }

  private updatePolling(): void {
    const shouldPoll = this.remoteMode
      && !this.snapshot.connected
      && this.snapshot.authenticated
      && !this.isOffline();
    if (!shouldPoll) {
      this.stopPolling();
      return;
    }
    if (this.pollTimer !== null) return;
    this.pollTimer = this.dependencies.setInterval(() => {
      void this.refreshDevices().catch(() => undefined);
    }, DEVICE_POLL_INTERVAL_MS);
  }

  private stopPolling(): void {
    if (this.pollTimer === null) return;
    this.dependencies.clearInterval(this.pollTimer);
    this.pollTimer = null;
  }

  private getClient(replaceForLogin = false): RelayClient {
    const relayUrl = this.getConfig().relayUrl;
    if (replaceForLogin || !this.client || this.clientUrl !== relayUrl) {
      this.finishConnection('Relay configuration changed');
      this.client = this.dependencies.createClient(relayUrl);
      this.clientUrl = relayUrl;
      if (!replaceForLogin) this.updateSnapshot({ authenticated: false });
    }
    return this.client;
  }

  private requireClient(): RelayClient {
    if (!this.snapshot.authenticated) throw new Error('Log in to the relay first');
    return this.getClient();
  }

  private assertOnline(): void {
    if (this.isOffline()) throw new Error('Remote access is disabled in offline mode');
  }

  private handleRequestError(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    const authenticated = !(typeof error === 'object' && error !== null && 'status' in error && error.status === 401);
    if (!authenticated) this.dependencies.saveAuthSession(null);
    this.updateSnapshot({ authenticated, error: message });
  }

  private updateSnapshot(patch: Partial<RemoteServiceSnapshot>): void {
    this.snapshot = { ...this.snapshot, ...patch };
    this.updatePolling();
    const snapshot = this.getSnapshot();
    for (const listener of this.listeners) listener(snapshot);
  }
}