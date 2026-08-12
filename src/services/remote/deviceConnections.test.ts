import * as assert from 'node:assert/strict';
import test from 'node:test';

import { DeviceConnectionManager } from './deviceConnections.ts';
import { PairedDeviceStore } from './pairedDeviceStore.ts';
import {
  encodeTerminalStreamFrame,
  TerminalStreamFrameDecoder,
} from './terminalStreamFrame.ts';
import type {
  IrohBiStream,
  IrohConnection,
  IrohEndpointAddr,
  IrohModule,
} from './irohStreams.ts';

const TICKET = 'endpointfaketicketaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const NODE_ID = 'node-1';

function makeFakeWorld() {
  const calls = {
    loads: 0,
    binds: 0,
    connects: [] as string[],
    generatedSeeds: 0,
    fromBytesSeeds: [] as number[][],
    boundSecrets: [] as number[][],
    presets: [] as string[],
    closedEndpoints: 0,
  };

  let resolveClosed: (reason: string) => void = () => {};
  const closedPromise = new Promise<string>((resolve) => {
    resolveClosed = resolve;
  });

  const fakeBi = (): IrohBiStream => {
    // Replies to the first frame (open -> opened, fsList -> fsListResult,
    // transferManifest -> transferAccepted, transferComplete ->
    // transferResult), like serve.rs's `serve_bi_stream` dispatch. A proper
    // waiter queue (not just "shift or hang forever") matters for transfer:
    // its round trip needs a *second* reply (transferResult) that is only
    // pushed after the reader has already made its first `read()` call and
    // found the queue empty in between.
    const items: number[][] = [];
    let waiter: (() => void) | null = null;
    const push = (item: number[]): void => {
      items.push(item);
      const resolve = waiter;
      waiter = null;
      resolve?.();
    };
    return {
      send: {
        async writeAll(bytes) {
          const decoder = new TerminalStreamFrameDecoder();
          decoder.push(Uint8Array.from(bytes));
          const frame = decoder.nextFrame();
          if (frame?.kind === 'open') {
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'opened',
                  payload: { sessionId: 'session-9', shell: '/bin/fake' },
                }),
              ),
            );
          }
          if (frame?.kind === 'fsList') {
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'fsListResult',
                  payload: { entries: [{ name: 'readme.md', isDirectory: false }] },
                }),
              ),
            );
          }
          if (frame?.kind === 'transferManifest') {
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'transferAccepted',
                  payload: { grantedBytes: 4 * 1024 * 1024 },
                }),
              ),
            );
          }
          if (frame?.kind === 'transferComplete') {
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'transferResult',
                  payload: { success: true, code: null, message: '' },
                }),
              ),
            );
          }
          if (frame?.kind === 'transferPullRequest') {
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'transferPullManifest',
                  payload: { entries: [{ index: 0, relativePath: 'a.md', size: 1 }] },
                }),
              ),
            );
          }
          if (frame?.kind === 'transferCredit') {
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'transferChunk',
                  payload: { fileIndex: 0, offset: 0, data: new TextEncoder().encode('a') },
                }),
              ),
            );
            push(
              Array.from(
                encodeTerminalStreamFrame({ kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 1 } }),
              ),
            );
            push(
              Array.from(
                encodeTerminalStreamFrame({
                  kind: 'transferResult',
                  payload: { success: true, code: null, message: '' },
                }),
              ),
            );
          }
        },
        async finish() {},
      },
      recv: {
        async read() {
          if (items.length === 0) {
            await new Promise<void>((resolve) => {
              waiter = resolve;
            });
          }
          return items.shift();
        },
      },
    };
  };

  const connection: IrohConnection & { biOpened: number } = {
    biOpened: 0,
    async openBi() {
      connection.biOpened += 1;
      return fakeBi();
    },
    close() {},
    closed: () => closedPromise,
  };

  let failNextConnect: string | null = null;

  const module: IrohModule = {
    Endpoint: {
      builder() {
        calls.binds += 1;
        return {
          secretKey(bytes: number[]) {
            calls.boundSecrets.push([...bytes]);
          },
          alpns() {},
          relayMode() {},
          bindAddr() {},
          async bind() {
            return {
              async connect(addr: IrohEndpointAddr) {
                calls.connects.push(addr.id().toString());
                if (failNextConnect) {
                  const message = failNextConnect;
                  failNextConnect = null;
                  throw new Error(message);
                }
                return connection;
              },
              async close() {
                calls.closedEndpoints += 1;
              },
            };
          },
        };
      },
    },
    EndpointTicket: {
      fromString(s: string) {
        assert.equal(s, TICKET, 'must dial the stored ticket');
        return { endpointAddr: () => ({ id: () => ({ toString: () => 'remote-id' }) }) };
      },
    },
    RelayMode: { disabled: () => 'disabled', defaultMode: () => 'default' },
    SecretKey: {
      generate() {
        calls.generatedSeeds += 1;
        return { toBytes: () => [7, 7, 7] };
      },
      fromBytes(bytes: number[]) {
        calls.fromBytesSeeds.push([...bytes]);
        return { toBytes: () => [...bytes] };
      },
    },
    presetN0() {
      calls.presets.push('n0');
    },
    presetMinimal() {
      calls.presets.push('minimal');
    },
  };

  const store = new PairedDeviceStore();
  store.upsert({ name: 'dev', ticket: TICKET, nodeId: NODE_ID });

  return {
    calls,
    store,
    module,
    connection,
    failConnectWith: (message: string) => {
      failNextConnect = message;
    },
    closeConnection: (reason: string) => resolveClosed(reason),
  };
}

function makeManager(world: ReturnType<typeof makeFakeWorld>, overrides = {}) {
  const created: number[][] = [];
  const manager = new DeviceConnectionManager({
    loadIroh: async () => {
      world.calls.loads += 1;
      return world.module;
    },
    store: world.store,
    identitySeed: null,
    onIdentityCreated: (seed) => created.push(seed),
    profile: 'loopback',
    now: () => '2026-07-31T10:00:00Z',
    ...overrides,
  });
  return { manager, created };
}

test('connect lazily binds one endpoint, creates and reports a new identity', async () => {
  const world = makeFakeWorld();
  const { manager, created } = makeManager(world);

  await manager.connect(NODE_ID);
  await manager.connect(NODE_ID); // idempotent while connected

  assert.equal(world.calls.loads, 1);
  assert.equal(world.calls.binds, 1, 'one endpoint for every connection');
  assert.deepEqual(created, [[7, 7, 7]], 'fresh identity must be reported for persisting');
  assert.deepEqual(world.calls.boundSecrets, [[7, 7, 7]]);
  assert.deepEqual(world.calls.presets, ['minimal']);
});

test('a persisted identity seed is reused instead of generating a new one', async () => {
  const world = makeFakeWorld();
  const { manager, created } = makeManager(world, { identitySeed: [1, 2, 3] });

  await manager.connect(NODE_ID);

  assert.equal(world.calls.generatedSeeds, 0);
  assert.deepEqual(world.calls.fromBytesSeeds, [[1, 2, 3]]);
  assert.deepEqual(created, []);
});

test('a successful connect records the connection history on the device', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);

  const changes: string[] = [];
  manager.onDidChange(() => changes.push(manager.status(NODE_ID).state));

  await manager.connect(NODE_ID);

  assert.equal(manager.isConnected(NODE_ID), true);
  const device = world.store.get(NODE_ID);
  assert.equal(device?.lastConnectedAt, '2026-07-31T10:00:00Z');
  assert.equal(device?.lastKnownOnline, true);
  assert.deepEqual(changes, ['connecting', 'connected']);
});

test('connecting to an unpaired device is refused', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);
  await assert.rejects(manager.connect('ghost'), /pair it first/);
});

test('a failed dial surfaces as an error status and marks the device offline', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);
  world.failConnectWith('no route to device');

  await assert.rejects(manager.connect(NODE_ID));

  assert.deepEqual(manager.status(NODE_ID), {
    state: 'error',
    code: 'CONNECT_FAILED',
    message: 'no route to device',
  });
  assert.equal(world.store.get(NODE_ID)?.lastKnownOnline, false);

  // The failure is not sticky: the next attempt may proceed.
  await manager.connect(NODE_ID);
  assert.equal(manager.isConnected(NODE_ID), true);
});

test('the connection dying flips the device offline', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);
  await manager.connect(NODE_ID);

  world.closeConnection('connection lost');
  await new Promise((resolve) => setTimeout(resolve, 10));

  assert.deepEqual(manager.status(NODE_ID), { state: 'disconnected' });
  assert.equal(world.store.get(NODE_ID)?.lastKnownOnline, false);
});

test('a CONTROLLER_ALREADY_CONNECTED close becomes a distinct error status', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);
  await manager.connect(NODE_ID);

  world.closeConnection('closed by peer: CONTROLLER_ALREADY_CONNECTED');
  await new Promise((resolve) => setTimeout(resolve, 10));

  const status = manager.status(NODE_ID);
  assert.equal(status.state, 'error');
  if (status.state === 'error') assert.equal(status.code, 'CONTROLLER_ALREADY_CONNECTED');
});

test('createTerminalTransport opens sessions on the device connection', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);

  assert.throws(() => manager.createTerminalTransport(NODE_ID), /connect to the device/);

  await manager.connect(NODE_ID);
  const transport = manager.createTerminalTransport(NODE_ID);
  const info = await transport.open({ cols: 80, rows: 24 });

  assert.deepEqual(info, { sessionId: 'session-9', shell: '/bin/fake' });
  assert.equal(world.connection.biOpened, 1, 'one bi-stream per session');
});

test('createDirectoryTreeSource lists a remote directory over the device connection', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);

  assert.throws(() => manager.createDirectoryTreeSource(NODE_ID), /connect to the device/);

  await manager.connect(NODE_ID);
  const source = manager.createDirectoryTreeSource(NODE_ID);
  const entries = await source.list('/home/user/project');

  assert.deepEqual(entries, [{ name: 'readme.md', isDirectory: false }]);
  assert.equal(world.connection.biOpened, 1, 'one bi-stream per list() call');
});

test('createTransferSender sends a note and resolves success over the device connection', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);

  assert.throws(
    () => manager.createTransferSender(NODE_ID, 't1', [{ index: 0, relativePath: 'a.md', size: 1 }], async () => new Uint8Array()),
    /connect to the device/,
  );

  await manager.connect(NODE_ID);
  const sender = manager.createTransferSender(
    NODE_ID,
    't1',
    [{ index: 0, relativePath: 'a.md', size: 1 }],
    async () => new TextEncoder().encode('a'),
  );
  const outcome = await sender.run();

  assert.deepEqual(outcome, { success: true, code: null, message: '' });
  assert.equal(world.connection.biOpened, 1, 'one bi-stream per transfer');
});

test('createTransferPuller pulls a file and resolves with its content over the device connection', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);

  assert.throws(() => manager.createTransferPuller(NODE_ID, '/home/user/a.md'), /connect to the device/);

  await manager.connect(NODE_ID);
  const puller = manager.createTransferPuller(NODE_ID, '/home/user/a.md');
  const outcome = await puller.run();

  assert.equal(outcome.success, true);
  assert.equal(outcome.files.length, 1);
  assert.equal(outcome.files[0].relativePath, 'a.md');
  assert.equal(new TextDecoder().decode(outcome.files[0].data), 'a');
  assert.equal(world.connection.biOpened, 1, 'one bi-stream per pull');
});

test('disconnect and dispose tear things down', async () => {
  const world = makeFakeWorld();
  const { manager } = makeManager(world);
  await manager.connect(NODE_ID);

  manager.disconnect(NODE_ID);
  assert.deepEqual(manager.status(NODE_ID), { state: 'disconnected' });
  assert.equal(world.store.get(NODE_ID)?.lastKnownOnline, false);

  await manager.dispose();
  assert.equal(world.calls.closedEndpoints, 1);
  await assert.rejects(manager.connect(NODE_ID), /disposed/);
});
