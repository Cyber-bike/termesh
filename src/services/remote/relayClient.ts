import WebSocket, { type RawData } from 'ws';

import {
  CONTROL_MESSAGE_TYPES,
  type ControlMessage,
} from '../../protocol/generated/messages.ts';
import type { ControlChannel } from './transferSender.ts';
import { toDisposable, type Disposable } from './transport.ts';

const CONTROL_SUBPROTOCOL = 'termy.v1';
const CONTROL_PATH = '/v1/control/ws';

export interface LoginResponse {
  accessToken: string;
  tokenType: 'Bearer';
  expiresIn: 900;
  user: { id: string; login: string };
}

export interface PairingCodeCreated {
  pairingCodeId: string;
  pairingCode: string;
  createdAt: string;
  revoked: false;
}

export interface Device {
  id: string;
  name: string;
  platform: 'windows-x64' | 'ubuntu-x64';
  agentVersion: string;
  online: boolean;
  lastSeenAt: string | null;
}

export interface DeviceList {
  devices: Device[];
}

export interface RelayErrorBody {
  error: { code: string; message: string; requestId: string };
}

export class RelayRequestError extends Error {
  readonly status: number;
  readonly code: string;
  readonly requestId: string | null;

  constructor(status: number, code: string, message: string, requestId: string | null = null) {
    super(message);
    this.name = 'RelayRequestError';
    this.status = status;
    this.code = code;
    this.requestId = requestId;
  }
}

export interface ControlCloseEvent {
  code: number;
  reason: string;
}

export interface RelayControlConnection extends ControlChannel, Disposable {
  sendJson(message: unknown): void;
  sendBinary(frame: Uint8Array): void;
  close(code?: number, reason?: string): void;
  onControlMessage(handler: (message: ControlMessage) => void): Disposable;
  onBinary(handler: (data: Uint8Array) => void): Disposable;
  onClose(handler: (event: ControlCloseEvent) => void): Disposable;
  onError(handler: (error: Error) => void): Disposable;
}

interface WebSocketLike {
  readonly readyState: number;
  on(event: 'open', listener: () => void): this;
  on(event: 'message', listener: (data: RawData, isBinary: boolean) => void): this;
  on(event: 'close', listener: (code: number, reason: Buffer) => void): this;
  on(event: 'error', listener: (error: Error) => void): this;
  send(data: string | Uint8Array): void;
  close(code?: number, reason?: string): void;
}

export interface RelayClientDependencies {
  fetch: (input: URL, init: RequestInit) => Promise<RelayHttpResponse>;
  now: () => number;
  createWebSocket: (
    url: string,
    protocol: string,
    headers: Record<string, string>
  ) => WebSocketLike;
}

interface RelayHttpResponse {
  readonly ok: boolean;
  readonly status: number;
  json(): Promise<unknown>;
}

const defaultDependencies: RelayClientDependencies = {
  fetch: requestWithObsidian,
  now: Date.now,
  createWebSocket: (url, protocol, headers) => new WebSocket(url, protocol, { headers }),
};

export class RelayClient {
  private readonly baseUrl: URL;
  private readonly dependencies: RelayClientDependencies;
  private accessToken: string | null = null;
  private expiresAt = 0;

  constructor(baseUrl: string, dependencies: Partial<RelayClientDependencies> = {}) {
    this.baseUrl = normalizeBaseUrl(baseUrl);
    this.dependencies = { ...defaultDependencies, ...dependencies };
  }

  async login(login: string, password: string): Promise<LoginResponse> {
    const response = await this.request<LoginResponse>('/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({ login, password }),
    }, false);
    this.accessToken = response.accessToken;
    this.expiresAt = this.dependencies.now() + response.expiresIn * 1000;
    return response;
  }

  async createPairingCode(): Promise<PairingCodeCreated> {
    return this.request('/v1/devices/pairing-codes', {
      method: 'POST',
      body: '{}',
    });
  }

  async revokePairingCode(id: string): Promise<void> {
    await this.request(`/v1/devices/pairing-codes/${encodeURIComponent(id)}`, { method: 'DELETE' });
  }

  async listDevices(): Promise<DeviceList> {
    return this.request('/v1/devices');
  }

  async deleteDevice(id: string): Promise<void> {
    await this.request(`/v1/devices/${encodeURIComponent(id)}`, { method: 'DELETE' });
  }

  async connectControl(): Promise<RelayControlConnection> {
    const token = this.requireAccessToken();
    const url = new URL(CONTROL_PATH, this.baseUrl);
    url.protocol = this.baseUrl.protocol === 'http:' ? 'ws:' : 'wss:';
    const socket = this.dependencies.createWebSocket(url.toString(), CONTROL_SUBPROTOCOL, {
      Authorization: `Bearer ${token}`,
    });

    return new Promise((resolve, reject) => {
      let settled = false;
      socket.on('open', () => {
        settled = true;
        resolve(new WebSocketControlConnection(socket));
      });
      socket.on('error', (error) => {
        if (!settled) {
          settled = true;
          reject(error);
        }
      });
      socket.on('close', (code, reason) => {
        if (!settled) {
          settled = true;
          reject(new Error(`Relay control connection closed during handshake (${code}): ${reason.toString()}`));
        }
      });
    });
  }

  private requireAccessToken(): string {
    if (this.accessToken === null || this.dependencies.now() >= this.expiresAt) {
      this.clearAuthentication();
      throw new RelayRequestError(401, 'AUTH_EXPIRED', 'Relay login has expired');
    }
    return this.accessToken;
  }

  private clearAuthentication(): void {
    this.accessToken = null;
    this.expiresAt = 0;
  }

  private async request<T>(
    path: string,
    init: RequestInit = {},
    authenticated = true
  ): Promise<T> {
    const headers = new Headers(init.headers);
    headers.set('Accept', 'application/json');
    if (init.body !== undefined) headers.set('Content-Type', 'application/json');
    if (authenticated) headers.set('Authorization', `Bearer ${this.requireAccessToken()}`);

    const response = await this.dependencies.fetch(new URL(path, this.baseUrl), { ...init, headers });
    if (!response.ok) {
      if (response.status === 401 && authenticated) this.clearAuthentication();
      throw await relayRequestError(response);
    }
    if (response.status === 204) return undefined as T;
    return response.json() as Promise<T>;
  }
}

class WebSocketControlConnection implements RelayControlConnection {
  private readonly socket: WebSocketLike;
  private readonly controlHandlers = new Set<(message: ControlMessage) => void>();
  private readonly binaryHandlers = new Set<(data: Uint8Array) => void>();
  private readonly closeHandlers = new Set<(event: ControlCloseEvent) => void>();
  private readonly errorHandlers = new Set<(error: Error) => void>();
  private closed = false;

  constructor(socket: WebSocketLike) {
    this.socket = socket;
    socket.on('message', (data, isBinary) => this.handleMessage(data, isBinary));
    socket.on('close', (code, reason) => {
      this.closed = true;
      const event = { code, reason: reason.toString() };
      for (const handler of this.closeHandlers) handler(event);
    });
    socket.on('error', (error) => {
      for (const handler of this.errorHandlers) handler(error);
    });
  }

  sendJson(message: unknown): void {
    this.ensureOpen();
    this.socket.send(JSON.stringify(message));
  }

  sendBinary(frame: Uint8Array): void {
    this.ensureOpen();
    this.socket.send(frame);
  }

  close(code = 1000, reason = 'client closed'): void {
    if (this.closed) return;
    this.closed = true;
    this.socket.close(code, reason);
  }

  dispose(): void {
    this.close();
    this.controlHandlers.clear();
    this.binaryHandlers.clear();
    this.closeHandlers.clear();
    this.errorHandlers.clear();
  }

  onControlMessage(handler: (message: ControlMessage) => void): Disposable {
    return subscribe(this.controlHandlers, handler);
  }

  onBinary(handler: (data: Uint8Array) => void): Disposable {
    return subscribe(this.binaryHandlers, handler);
  }

  onClose(handler: (event: ControlCloseEvent) => void): Disposable {
    return subscribe(this.closeHandlers, handler);
  }

  onError(handler: (error: Error) => void): Disposable {
    return subscribe(this.errorHandlers, handler);
  }

  private ensureOpen(): void {
    if (this.closed || this.socket.readyState !== WebSocket.OPEN) {
      throw new Error('Relay control connection is not open');
    }
  }

  private handleMessage(data: RawData, isBinary: boolean): void {
    if (isBinary) {
      const bytes = rawDataToBytes(data);
      for (const handler of this.binaryHandlers) handler(bytes);
      return;
    }

    try {
      const value: unknown = JSON.parse(rawDataToText(data));
      if (!isControlMessage(value)) throw new Error('Unknown relay control message');
      for (const handler of this.controlHandlers) handler(value);
    } catch (error) {
      const normalized = error instanceof Error ? error : new Error(String(error));
      for (const handler of this.errorHandlers) handler(normalized);
    }
  }
}

function normalizeBaseUrl(baseUrl: string): URL {
  const url = new URL(baseUrl);
  if (url.protocol !== 'https:') {
    throw new Error('Relay URL must use HTTPS');
  }
  url.pathname = '/';
  url.search = '';
  url.hash = '';
  return url;
}

async function relayRequestError(response: RelayHttpResponse): Promise<RelayRequestError> {
  let body: RelayErrorBody | null = null;
  try {
    body = await response.json() as RelayErrorBody;
  } catch {
    // Preserve the HTTP status when a proxy returns a non-JSON error page.
  }
  return new RelayRequestError(
    response.status,
    body?.error.code ?? 'HTTP_ERROR',
    body?.error.message ?? `Relay request failed with HTTP ${response.status}`,
    body?.error.requestId ?? null
  );
}

function isControlMessage(value: unknown): value is ControlMessage {
  if (typeof value !== 'object' || value === null || !('type' in value)) return false;
  const type = (value as { type?: unknown }).type;
  return typeof type === 'string' && (CONTROL_MESSAGE_TYPES as readonly string[]).includes(type);
}

function rawDataToBytes(data: RawData): Uint8Array {
  if (typeof data === 'string') return new TextEncoder().encode(data);
  if (Array.isArray(data)) return Buffer.concat(data);
  return new Uint8Array(data instanceof ArrayBuffer ? data : data.buffer, data instanceof ArrayBuffer ? 0 : data.byteOffset, data instanceof ArrayBuffer ? data.byteLength : data.byteLength);
}

function rawDataToText(data: RawData): string {
  return typeof data === 'string' ? data : new TextDecoder().decode(rawDataToBytes(data));
}

function subscribe<T>(handlers: Set<T>, handler: T): Disposable {
  handlers.add(handler);
  return toDisposable(() => handlers.delete(handler));
}

async function requestWithObsidian(input: URL, init: RequestInit): Promise<RelayHttpResponse> {
  const { requestUrl } = await import('obsidian');
  const headers: Record<string, string> = {};
  new Headers(init.headers).forEach((value, key) => {
    headers[key] = value;
  });
  const response = await requestUrl({
    url: input.toString(),
    method: init.method,
    body: typeof init.body === 'string' ? init.body : undefined,
    headers,
    throw: false,
  });
  return {
    ok: response.status >= 200 && response.status < 300,
    status: response.status,
    json: () => Promise.resolve(response.json as unknown),
  };
}