import * as assert from 'node:assert/strict';
import test from 'node:test';

import { pairDevice } from './devicePairing.ts';
import { PairedDeviceStore } from './pairedDeviceStore.ts';

/** Same real `@number0/iroh` 1.1.0 ticket as connectionCode.test.ts. */
const REAL_TICKET =
  'endpointactqqssc2bihkta43whj3xhgxbf6nq75f7wxi6zdldbgn65cjllpeaibab7qaaabxhjag';

const NODE_ID = 'actqqssc2bihkta43whj3xhgxbf6nq75f7wxi6zdldbgn65cjllq';

function fakeParser(code: string): { nodeId: string } {
  assert.equal(code, REAL_TICKET, 'the parser must receive the normalized code');
  return { nodeId: NODE_ID };
}

test('a valid code lands in the store with the user-given name', () => {
  const store = new PairedDeviceStore();
  const result = pairDevice(store, fakeParser, `  ${REAL_TICKET.toUpperCase()}  `, 'build-server');

  assert.equal(result.ok, true);
  if (result.ok) {
    assert.equal(result.device.name, 'build-server');
    assert.equal(result.device.nodeId, NODE_ID);
    assert.equal(result.device.ticket, REAL_TICKET, 'the normalized ticket is stored, not the raw paste');
  }
  assert.equal(store.count(), 1);
});

test('a blank name falls back to a short id prefix', () => {
  const store = new PairedDeviceStore();
  const result = pairDevice(store, fakeParser, REAL_TICKET, '   ');

  assert.equal(result.ok, true);
  if (result.ok) assert.equal(result.device.name, NODE_ID.slice(0, 10));
});

test('a code failing pre-validation never reaches the parser or the store', () => {
  const store = new PairedDeviceStore();
  const parser = () => {
    throw new Error('must not be called');
  };

  const result = pairDevice(store, parser, 'not-a-ticket', 'x');
  assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'wrong-prefix' });
  assert.equal(store.count(), 0);
});

test('a parser rejection maps to TICKET_INVALID/unparseable and stores nothing', () => {
  const store = new PairedDeviceStore();
  const parser = () => {
    throw new Error('bad ticket payload');
  };

  const result = pairDevice(store, parser, REAL_TICKET, 'x');
  assert.deepEqual(result, { ok: false, code: 'TICKET_INVALID', problem: 'unparseable' });
  assert.equal(store.count(), 0);
});

test('re-pairing the same device updates the entry instead of duplicating it', () => {
  const store = new PairedDeviceStore();
  pairDevice(store, fakeParser, REAL_TICKET, 'old-name');
  store.markConnected(NODE_ID, '2026-07-30T09:00:00Z');

  const result = pairDevice(store, fakeParser, REAL_TICKET, 'new-name');

  assert.equal(result.ok, true);
  assert.equal(store.count(), 1);
  const device = store.get(NODE_ID);
  assert.equal(device?.name, 'new-name');
  assert.equal(device?.lastConnectedAt, '2026-07-30T09:00:00Z', 'history survives re-pairing');
});
