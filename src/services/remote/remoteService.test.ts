import * as assert from 'node:assert/strict';
import test from 'node:test';

import type { ControlMessage } from '../../protocol/generated/messages.ts';
import type { Device, RelayClient, RelayControlConnection } from './relayClient.ts';
import { RemoteService } from './remoteService.ts';
import { toDisposable, type Disposable } from './transport.ts';

const DEVICE_ID = '3d594650-3436-4c7a-9a15-9b5c3f0f4a11';
const TRANSFER_ID = '7d594650-3436-4c7a-9a15-9b5c3f0f4a12';
const REQUEST_ID = 'e2f3a4b5-c6d7-4e8f-9a0b-1c2d3e4f5a6b';

class MockConnection implements RelayControlConnection {
  json: unknown[] = [];
  binary: Uint8Array[] = [];
  controls = new Set<(message: ControlMessage) => void>();
  closes = new Set<(event: { code: number; reason: string }) => void>();
  errors = new Set<(error: Error) => void>();

  sendJson(message: unknown): void { this.json.push(message); }
  sendBinary(frame: Uint8Array): void { this.binary.push(frame); }
  close(): void {}
  dispose(): void {}
  onControlMessage(handler: (message: ControlMessage) => void): Disposable { return subscribe(this.controls, handler); }
  onBinary(): Disposable { return toDisposable(() => undefined); }
  onClose(handler: (event: { code: number; reason: string }) => void): Disposable { return subscribe(this.closes, handler); }
  onError(handler: (error: Error) => void): Disposable { return subscribe(this.errors, handler); }
  emit(message: ControlMessage): void { for (const handler of this.controls) handler(message); }
}

class MockClient {
  loginCalls = 0;
  listCalls = 0;
  restoredToken: string | null = null;
  connection = new MockConnection();
  devices: Device[] = [{ id: DEVICE_ID, name: 'Example', platform: 'windows-x64', agentVersion: '1.0.0', online: true, lastSeenAt: null }];

  async login(): Promise<{ accessToken: string; tokenType: 'Bearer'; expiresIn: number; user: { id: string; login: string } }> {
    this.loginCalls += 1;
    return { accessToken: 'token', tokenType: 'Bearer', expiresIn: 900, user: { id: DEVICE_ID, login: 'example' } };
  }
  async register(): Promise<{ accessToken: string; tokenType: 'Bearer'; expiresIn: number; user: { id: string; login: string } }> {
    return this.login();
  }
  restoreAuthentication(session: { accessToken: string }): void { this.restoredToken = session.accessToken; }
  async listDevices(): Promise<{ devices: Device[] }> { this.listCalls += 1; return { devices: this.devices }; }
  async connectControl(): Promise<RelayControlConnection> { return this.connection; }
  async createPairingCode(): Promise<never> { throw new Error('unused'); }
  async revokePairingCode(): Promise<void> {}
  async deleteDevice(): Promise<void> {}
}

function subscribe<T>(handlers: Set<T>, handler: T): Disposable {
  handlers.add(handler);
  return toDisposable(() => handlers.delete(handler));
}

function createService(
  client: MockClient,
  offline = false,
  authSession: { accessToken: string; expiresAt: number; login: string } | null = null,
) {
  let intervalHandler: (() => void) | null = null;
  let intervalMs = 0;
  const ids = [TRANSFER_ID, REQUEST_ID];
  const service = new RemoteService(
    () => ({ relayUrl: 'https://relay.example.com', deviceId: DEVICE_ID, authSession }),
    () => offline,
    {
      createClient: () => client as unknown as RelayClient,
      createId: () => ids.shift() ?? crypto.randomUUID(),
      setInterval: (handler, timeoutMs) => { intervalHandler = handler; intervalMs = timeoutMs; return 1; },
      clearInterval: () => { intervalHandler = null; },
      setTimeout: (handler) => Number(setTimeout(handler, 1000)),
      clearTimeout: (timer) => clearTimeout(timer),
    },
  );
  return { service, getInterval: () => ({ handler: intervalHandler, ms: intervalMs }) };
}

test('offline mode blocks login before creating a relay client', async () => {
  const client = new MockClient();
  const { service } = createService(client, true);
  await assert.rejects(service.login('example', 'password'), /offline mode/);
  assert.equal(client.loginCalls, 0);
});

test('persisted authentication is restored and validated by loading devices', async () => {
  const client = new MockClient();
  const { service } = createService(client, false, {
    accessToken: 'persisted-token',
    expiresAt: Date.now() + 60_000,
    login: 'example',
  });

  assert.equal(await service.restoreAuthentication(), true);
  assert.equal(client.restoredToken, 'persisted-token');
  assert.equal(client.listCalls, 1);
  assert.equal(service.getSnapshot().authenticated, true);
});

test('device polling runs every 15 seconds only in remote disconnected mode', async () => {
  const client = new MockClient();
  const { service, getInterval } = createService(client);
  await service.login('example', 'password');
  assert.equal(getInterval().handler, null);
  service.setRemoteMode(true);
  assert.equal(getInterval().ms, 15_000);
  assert.ok(getInterval().handler);
  await service.connect();
  assert.equal(getInterval().handler, null);
});

test('transfer waits for accepted credit and routes credit and result', async () => {
  const client = new MockClient();
  const { service } = createService(client);
  await service.login('example', 'password');
  await service.connect();

  const running = service.transfer({
    rootNotePath: 'notes/demo.md',
    sizeOf: () => 1,
    links: () => [],
  }, async () => new Uint8Array([65]));

  const start = client.connection.json[0] as { requestId: string; payload: { transferId: string } };
  client.connection.emit({
    protocolVersion: 1,
    type: 'transfer.accepted',
    requestId: start.requestId,
    deviceId: DEVICE_ID,
    sessionId: null,
    payload: { transferId: start.payload.transferId, grantedBytes: 1 },
  });
  await new Promise<void>((resolve) => setImmediate(resolve));
  client.connection.emit({
    protocolVersion: 1,
    type: 'transfer.credit',
    requestId: null,
    deviceId: DEVICE_ID,
    sessionId: null,
    payload: { transferId: start.payload.transferId, grantedBytes: 2 },
  });
  client.connection.emit({
    protocolVersion: 1,
    type: 'transfer.result',
    requestId: null,
    deviceId: DEVICE_ID,
    sessionId: null,
    payload: {
      transferId: start.payload.transferId,
      success: true,
      code: null,
      message: 'ok',
      destinationPath: 'C:\\Users\\example\\TermyReceive\\notes\\demo.md',
    },
  });

  assert.deepEqual(await running, {
    success: true,
    code: null,
    message: 'ok',
    destinationPath: 'C:\\Users\\example\\TermyReceive\\notes\\demo.md',
  });
  assert.equal((client.connection.json[1] as { type: string }).type, 'transfer.fileEnd');
  assert.equal((client.connection.json[2] as { type: string }).type, 'transfer.complete');
});
