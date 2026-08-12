import * as assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';

import { decodeFrame, encodeFrame, KIND_TERMINAL_OUTPUT, TERMINAL_FILE_INDEX } from './frameCodec.ts';
import {
  RelayClient,
  RelayRequestError,
  type RelayClientDependencies,
} from './relayClient.ts';

const TOKEN = 'test-access-token';
const SESSION = '7d594650-3436-4c7a-9a15-9b5c3f0f4a12';

class MockSocket extends EventEmitter {
  readyState = 1;
  sent: Array<string | Uint8Array> = [];
  closeArgs: [number | undefined, string | undefined] | null = null;

  send(data: string | Uint8Array): void {
    this.sent.push(data);
  }

  close(code?: number, reason?: string): void {
    this.closeArgs = [code, reason];
  }
}

function loginResponse(): Response {
  return Response.json({
    accessToken: TOKEN,
    tokenType: 'Bearer',
    expiresIn: 900,
    user: { id: '3d594650-3436-4c7a-9a15-9b5c3f0f4a11', login: 'example' },
  });
}

test('HTTP methods attach Bearer auth and use the documented routes', async () => {
  const calls: Array<{ url: string; init: RequestInit }> = [];
  const responses = [
    loginResponse(),
    loginResponse(),
    Response.json({ pairingCodeId: SESSION, pairingCode: 'a'.repeat(27), createdAt: '2026-01-01T00:00:00Z', revoked: false }, { status: 201 }),
    new Response(null, { status: 204 }),
    Response.json({ devices: [] }),
    new Response(null, { status: 204 }),
  ];
  const client = new RelayClient('https://relay.example.com/base', {
    fetch: async (input, init) => {
      calls.push({ url: input.toString(), init: init ?? {} });
      const response = responses.shift();
      assert.ok(response);
      return response;
    },
    now: () => 1_000,
  });

  await client.login('example', 'password123');
  await client.register('new-example', 'password123');
  await client.createPairingCode();
  await client.revokePairingCode(SESSION);
  await client.listDevices();
  await client.deleteDevice(SESSION);

  assert.deepEqual(calls.map((call) => new URL(call.url).pathname), [
    '/v1/auth/login',
    '/v1/auth/register',
    '/v1/devices/pairing-codes',
    `/v1/devices/pairing-codes/${SESSION}`,
    '/v1/devices',
    `/v1/devices/${SESSION}`,
  ]);
  assert.equal(new Headers(calls[0].init.headers).get('Authorization'), null);
  assert.equal(new Headers(calls[1].init.headers).get('Authorization'), null);
  for (const call of calls.slice(2)) {
    assert.equal(new Headers(call.init.headers).get('Authorization'), `Bearer ${TOKEN}`);
  }
});

test('local expiry and a server 401 both require a new login', async () => {
  let now = 0;
  let calls = 0;
  const client = new RelayClient('https://relay.example.com', {
    now: () => now,
    fetch: async () => {
      calls += 1;
      return calls <= 2
        ? loginResponse()
        : Response.json({ error: { code: 'AUTH_EXPIRED', message: 'expired', requestId: 'request-1' } }, { status: 401 });
    },
  });

  await client.login('example', 'password123');
  now = 900_000;
  await assert.rejects(client.listDevices(), (error: unknown) => {
    assert.ok(error instanceof RelayRequestError);
    return error.code === 'AUTH_EXPIRED';
  });
  assert.equal(calls, 1, 'an expired token must not be sent');

  now = 0;
  await client.login('example', 'password123');
  await assert.rejects(client.listDevices(), { name: 'RelayRequestError' });
  await assert.rejects(client.listDevices(), (error: unknown) => {
    assert.ok(error instanceof RelayRequestError);
    return error.status === 401;
  });
  assert.equal(calls, 3, 'the request after a 401 must be blocked locally');
});

test('control WSS uses auth and subprotocol and dispatches text and binary', async () => {
  const socket = new MockSocket();
  let handshake: { url: string; protocol: string; headers: Record<string, string> } | null = null;
  const dependencies: Partial<RelayClientDependencies> = {
    fetch: async () => loginResponse(),
    now: () => 0,
    createWebSocket: (url, protocol, headers) => {
      handshake = { url, protocol, headers };
      return socket;
    },
  };
  const client = new RelayClient('https://relay.example.com', dependencies);
  await client.login('example', 'password123');

  const connecting = client.connectControl();
  socket.emit('open');
  const connection = await connecting;
  assert.deepEqual(handshake, {
    url: 'wss://relay.example.com/v1/control/ws',
    protocol: 'termy.v1',
    headers: { Authorization: `Bearer ${TOKEN}` },
  });

  const controls: string[] = [];
  const binaries: Uint8Array[] = [];
  connection.onControlMessage((message) => controls.push(message.type));
  connection.onBinary((data) => binaries.push(data));

  socket.emit('message', Buffer.from(JSON.stringify({
    protocolVersion: 1,
    type: 'terminal.close',
    requestId: null,
    deviceId: '3d594650-3436-4c7a-9a15-9b5c3f0f4a11',
    sessionId: SESSION,
    payload: { reason: 'shell_exited', exitCode: 0 },
  })), false);
  const encoded = encodeFrame({
    kind: KIND_TERMINAL_OUTPUT,
    streamId: SESSION,
    fileIndex: TERMINAL_FILE_INDEX,
    offset: 0,
    payload: new Uint8Array([1, 2, 3]),
  });
  socket.emit('message', Buffer.from(encoded), true);

  assert.deepEqual(controls, ['terminal.close']);
  assert.deepEqual(decodeFrame(binaries[0]).payload, new Uint8Array([1, 2, 3]));
  connection.close();
  assert.deepEqual(socket.closeArgs, [1000, 'client closed']);
});

test('control WSS retries once with a fresh socket after ECONNRESET', async () => {
  const firstSocket = new MockSocket();
  const secondSocket = new MockSocket();
  const sockets = [firstSocket, secondSocket];
  let attempts = 0;
  const client = new RelayClient('https://relay.example.com', {
    fetch: async () => loginResponse(),
    now: () => 0,
    createWebSocket: () => {
      attempts += 1;
      const socket = sockets.shift();
      assert.ok(socket);
      return socket;
    },
  });
  await client.login('example', 'password123');

  const connecting = client.connectControl();
  const reset = Object.assign(new Error('read ECONNRESET'), { code: 'ECONNRESET' });
  firstSocket.emit('error', reset);
  await new Promise<void>((resolve) => setImmediate(resolve));
  secondSocket.emit('open');

  const connection = await connecting;
  assert.equal(attempts, 2);
  assert.deepEqual(firstSocket.closeArgs, [undefined, undefined]);
  connection.sendJson({ type: 'ping' });
  assert.equal(secondSocket.sent.length, 1);
});

test('repeated ECONNRESET identifies the relay TLS and WebSocket checks', async () => {
  const firstSocket = new MockSocket();
  const secondSocket = new MockSocket();
  const sockets = [firstSocket, secondSocket];
  const client = new RelayClient('https://relay.example.com', {
    fetch: async () => loginResponse(),
    now: () => 0,
    createWebSocket: () => {
      const socket = sockets.shift();
      assert.ok(socket);
      return socket;
    },
  });
  await client.login('example', 'password123');

  const connecting = client.connectControl();
  const reset = Object.assign(new Error('read ECONNRESET'), { code: 'ECONNRESET' });
  firstSocket.emit('error', reset);
  await new Promise<void>((resolve) => setImmediate(resolve));
  secondSocket.emit('error', reset);

  await assert.rejects(connecting, (error: unknown) => {
    assert.ok(error instanceof Error);
    assert.equal((error as Error & { code?: string }).code, 'ECONNRESET');
    return /relay\.example\.com.*port 443.*WebSocket/s.test(error.message);
  });
});

test('non-HTTPS relay URLs are rejected', () => {
  assert.throws(() => new RelayClient('http://relay.example.com'), /HTTPS/);
});