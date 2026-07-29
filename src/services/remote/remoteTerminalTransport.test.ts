import * as assert from 'node:assert/strict';
import test from 'node:test';

import type { ControlMessage } from '../../protocol/generated/messages.ts';
import {
  decodeFrame,
  encodeFrame,
  KIND_TERMINAL_INPUT,
  KIND_TERMINAL_OUTPUT,
  TERMINAL_FILE_INDEX,
  TERMINAL_PAYLOAD_MAX,
} from './frameCodec.ts';
import type {
  ControlCloseEvent,
  RelayControlConnection,
} from './relayClient.ts';
import { RemoteTerminalTransport } from './remoteTerminalTransport.ts';
import { toDisposable, type Disposable } from './transport.ts';

const DEVICE = '3d594650-3436-4c7a-9a15-9b5c3f0f4a11';
const SESSION = '7d594650-3436-4c7a-9a15-9b5c3f0f4a12';
const REQUEST = 'e2f3a4b5-c6d7-4e8f-9a0b-1c2d3e4f5a6b';

const testTimers = {
  setTimeout: (handler: () => void, timeoutMs: number): number => Number(setTimeout(handler, timeoutMs)),
  clearTimeout: (timer: number): void => clearTimeout(timer),
};

class MockConnection implements RelayControlConnection {
  json: unknown[] = [];
  binary: Uint8Array[] = [];
  controls = new Set<(message: ControlMessage) => void>();
  binaries = new Set<(data: Uint8Array) => void>();
  closes = new Set<(event: ControlCloseEvent) => void>();
  errors = new Set<(error: Error) => void>();

  sendJson(message: unknown): void { this.json.push(message); }
  sendBinary(frame: Uint8Array): void { this.binary.push(frame); }
  close(): void {}
  dispose(): void {}
  onControlMessage(handler: (message: ControlMessage) => void): Disposable {
    return addHandler(this.controls, handler);
  }
  onBinary(handler: (data: Uint8Array) => void): Disposable {
    return addHandler(this.binaries, handler);
  }
  onClose(handler: (event: ControlCloseEvent) => void): Disposable {
    return addHandler(this.closes, handler);
  }
  onError(handler: (error: Error) => void): Disposable {
    return addHandler(this.errors, handler);
  }
  emitControl(message: ControlMessage): void {
    for (const handler of this.controls) handler(message);
  }
  emitBinary(data: Uint8Array): void {
    for (const handler of this.binaries) handler(data);
  }
}

function addHandler<T>(handlers: Set<T>, handler: T): Disposable {
  handlers.add(handler);
  return toDisposable(() => handlers.delete(handler));
}

function openedMessage(): ControlMessage {
  return {
    protocolVersion: 1,
    type: 'terminal.opened',
    requestId: REQUEST,
    deviceId: DEVICE,
    sessionId: SESSION,
    payload: { shell: 'pwsh' },
  };
}

async function openTransport(connection: MockConnection): Promise<RemoteTerminalTransport> {
  const transport = new RemoteTerminalTransport(connection, DEVICE, {
    createRequestId: () => REQUEST,
    ...testTimers,
  });
  const opening = transport.open({ cols: 120, rows: 40 });
  const sent = connection.json[0] as { type: string; requestId: string; payload: unknown };
  assert.deepEqual(sent, {
    protocolVersion: 1,
    type: 'terminal.open',
    requestId: REQUEST,
    deviceId: DEVICE,
    sessionId: null,
    payload: { cols: 120, rows: 40 },
  });
  connection.emitControl(openedMessage());
  assert.deepEqual(await opening, { sessionId: SESSION, shell: 'pwsh' });
  return transport;
}

test('open correlates terminal.opened and times out unanswered requests', async () => {
  const connection = new MockConnection();
  await openTransport(connection);

  const timedOut = new RemoteTerminalTransport(new MockConnection(), DEVICE, {
    createRequestId: () => REQUEST,
    openTimeoutMs: 1,
    ...testTimers,
  });
  await assert.rejects(timedOut.open({ cols: 80, rows: 24 }), /15 seconds/);
});

test('write sends kind 0x01 frames with contiguous offsets', async () => {
  const connection = new MockConnection();
  const transport = await openTransport(connection);
  const input = new Uint8Array(TERMINAL_PAYLOAD_MAX + 7).fill(9);

  transport.write(input);

  const frames = connection.binary.map(decodeFrame);
  assert.deepEqual(frames.map((frame) => frame.kind), [KIND_TERMINAL_INPUT, KIND_TERMINAL_INPUT]);
  assert.deepEqual(frames.map((frame) => frame.offset), [0, TERMINAL_PAYLOAD_MAX]);
  assert.deepEqual(frames.map((frame) => frame.payload.length), [TERMINAL_PAYLOAD_MAX, 7]);
});

test('output, shell, exit and error messages reach the four event channels', async () => {
  const connection = new MockConnection();
  const transport = await openTransport(connection);
  const data: number[][] = [];
  const shells: string[] = [];
  const exits: Array<number | null> = [];
  const errors: string[] = [];
  transport.onData((bytes) => data.push([...bytes]));
  transport.onShellEvent((event) => shells.push(event.type));
  transport.onExit((event) => exits.push(event.exitCode));
  transport.onError((code) => errors.push(code));

  connection.emitBinary(encodeFrame({
    kind: KIND_TERMINAL_OUTPUT,
    streamId: SESSION,
    fileIndex: TERMINAL_FILE_INDEX,
    offset: 0,
    payload: new Uint8Array([65, 66]),
  }));
  connection.emitControl({
    protocolVersion: 1,
    type: 'terminal.shellEvent',
    requestId: null,
    deviceId: DEVICE,
    sessionId: SESSION,
    payload: { type: 'command_end', source: 'osc633', exitCode: 0 },
  });
  connection.emitControl({
    protocolVersion: 1,
    type: 'terminal.error',
    requestId: null,
    deviceId: DEVICE,
    sessionId: SESSION,
    payload: { code: 'WRITE_FAILED', message: 'write failed' },
  });
  connection.emitControl({
    protocolVersion: 1,
    type: 'terminal.close',
    requestId: null,
    deviceId: DEVICE,
    sessionId: SESSION,
    payload: { reason: 'shell_exited', exitCode: 12 },
  });

  assert.deepEqual(data, [[65, 66]]);
  assert.deepEqual(shells, ['command_end']);
  assert.deepEqual(errors, ['WRITE_FAILED']);
  assert.deepEqual(exits, [12]);
});

test('resize and close send their control envelopes', async () => {
  const connection = new MockConnection();
  const transport = await openTransport(connection);

  transport.resize(90, 30);
  await transport.close();

  assert.deepEqual((connection.json[1] as { type: string; payload: unknown }), {
    protocolVersion: 1,
    type: 'terminal.resize',
    requestId: null,
    deviceId: DEVICE,
    sessionId: SESSION,
    payload: { cols: 90, rows: 30 },
  });
  assert.equal((connection.json[2] as { type: string }).type, 'terminal.close');
  assert.deepEqual((connection.json[2] as { payload: unknown }).payload, {
    reason: 'user',
    exitCode: null,
  });
});