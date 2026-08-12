import * as assert from 'node:assert/strict';
import test from 'node:test';

import { PairedDeviceStore } from './pairedDeviceStore.ts';

function sample(overrides: Partial<{ name: string; ticket: string; nodeId: string }> = {}) {
  return {
    name: 'build-server',
    ticket: 'nodeticket-abc123',
    nodeId: 'node-1',
    ...overrides,
  };
}

test('a new device starts with no connection history', () => {
  const store = new PairedDeviceStore();
  const device = store.upsert(sample());

  assert.equal(device.name, 'build-server');
  assert.equal(device.nodeId, 'node-1');
  assert.equal(device.lastConnectedAt, null);
  assert.equal(device.lastKnownOnline, false);
  assert.equal(store.count(), 1);
});

test('re-pairing the same nodeId updates name and ticket in place', () => {
  const store = new PairedDeviceStore();
  store.upsert(sample());
  store.markConnected('node-1', '2026-07-30T09:00:00Z');

  const updated = store.upsert(sample({ name: 'build-server-2', ticket: 'nodeticket-new' }));

  assert.equal(store.count(), 1, 'must update, not duplicate');
  assert.equal(updated.name, 'build-server-2');
  assert.equal(updated.ticket, 'nodeticket-new');
  assert.equal(updated.lastConnectedAt, '2026-07-30T09:00:00Z', 'connection history survives re-pairing');
});

test('blank fields are rejected', () => {
  const store = new PairedDeviceStore();
  assert.throws(() => store.upsert(sample({ name: '  ' })));
  assert.throws(() => store.upsert(sample({ ticket: '' })));
  assert.throws(() => store.upsert(sample({ nodeId: '   ' })));
});

test('fields are trimmed', () => {
  const store = new PairedDeviceStore();
  const device = store.upsert(sample({ name: '  build-server  ' }));
  assert.equal(device.name, 'build-server');
});

test('markConnected sets both the timestamp and online status', () => {
  const store = new PairedDeviceStore();
  store.upsert(sample());
  store.markConnected('node-1', '2026-07-30T09:00:00Z');

  const device = store.get('node-1');
  assert.equal(device?.lastConnectedAt, '2026-07-30T09:00:00Z');
  assert.equal(device?.lastKnownOnline, true);
});

test('setOnline toggles reachability without touching lastConnectedAt', () => {
  const store = new PairedDeviceStore();
  store.upsert(sample());
  store.markConnected('node-1', '2026-07-30T09:00:00Z');
  store.setOnline('node-1', false);

  const device = store.get('node-1');
  assert.equal(device?.lastKnownOnline, false);
  assert.equal(device?.lastConnectedAt, '2026-07-30T09:00:00Z');
});

test('operations on an unknown nodeId are harmless no-ops', () => {
  const store = new PairedDeviceStore();
  store.markConnected('ghost', '2026-07-30T09:00:00Z');
  store.setOnline('ghost', true);
  assert.equal(store.remove('ghost'), false);
  assert.equal(store.get('ghost'), undefined);
});

test('list preserves insertion order and returns independent copies', () => {
  const store = new PairedDeviceStore();
  store.upsert(sample({ nodeId: 'node-1', name: 'first' }));
  store.upsert(sample({ nodeId: 'node-2', name: 'second' }));

  const list = store.list();
  assert.deepEqual(list.map((d) => d.nodeId), ['node-1', 'node-2']);

  list[0].name = 'mutated';
  assert.equal(store.get('node-1')?.name, 'first', 'list() must not expose internal state');
});

test('remove drops exactly one device', () => {
  const store = new PairedDeviceStore();
  store.upsert(sample({ nodeId: 'node-1' }));
  store.upsert(sample({ nodeId: 'node-2' }));

  assert.equal(store.remove('node-1'), true);
  assert.equal(store.count(), 1);
  assert.equal(store.has('node-1'), false);
  assert.equal(store.has('node-2'), true);
});

test('round-trips through JSON', () => {
  const store = new PairedDeviceStore();
  store.upsert(sample({ nodeId: 'node-1' }));
  store.markConnected('node-1', '2026-07-30T09:00:00Z');

  const restored = PairedDeviceStore.fromJSON(store.toJSON());
  assert.deepEqual(restored.list(), store.list());
});

test('fromJSON tolerates non-array input', () => {
  assert.equal(PairedDeviceStore.fromJSON(null).count(), 0);
  assert.equal(PairedDeviceStore.fromJSON(undefined).count(), 0);
  assert.equal(PairedDeviceStore.fromJSON({ not: 'an array' }).count(), 0);
});

test('fromJSON skips malformed entries but keeps the valid ones', () => {
  const raw = [
    { name: 'good', ticket: 't1', nodeId: 'n1', lastConnectedAt: null, lastKnownOnline: false },
    { name: 'missing-ticket', nodeId: 'n2' },
    { name: '', ticket: 't3', nodeId: 'n3' },
    'not even an object',
    null,
    { name: 'good-2', ticket: 't4', nodeId: 'n4' },
  ];

  const store = PairedDeviceStore.fromJSON(raw);
  assert.deepEqual(
    store.list().map((d) => d.nodeId),
    ['n1', 'n4']
  );
});

test('fromJSON defaults a missing lastKnownOnline to false and non-string lastConnectedAt to null', () => {
  const raw = [{ name: 'n', ticket: 't', nodeId: 'id' }];
  const store = PairedDeviceStore.fromJSON(raw);
  const device = store.get('id');
  assert.equal(device?.lastKnownOnline, false);
  assert.equal(device?.lastConnectedAt, null);
});
