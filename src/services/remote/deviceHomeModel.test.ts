import * as assert from 'node:assert/strict';
import test from 'node:test';

import { buildDeviceHomeCards, getRefreshNodeIds } from './deviceHomeModel.ts';
import type { PairedDevice } from './pairedDeviceStore.ts';

function device(name: string, nodeId: string): PairedDevice {
  return {
    name,
    nodeId,
    ticket: `endpoint-${nodeId}`,
    lastConnectedAt: null,
    lastKnownOnline: false,
  };
}

test('orders add and local cards before paired devices', () => {
  const cards = buildDeviceHomeCards(
    [device('One', 'node-1'), device('Two', 'node-2')],
    { status: () => ({ state: 'disconnected' }) },
  );

  assert.deepEqual(cards.map((card) => card.kind), ['add', 'local', 'remote', 'remote']);
  assert.equal(cards[2]?.kind === 'remote' ? cards[2].device.name : null, 'One');
});

test('refreshes only remote devices that are not connected or connecting', () => {
  const cards = buildDeviceHomeCards(
    [device('One', 'node-1'), device('Two', 'node-2'), device('Three', 'node-3')],
    {
      status: (nodeId) => nodeId === 'node-1'
        ? { state: 'connected' }
        : nodeId === 'node-2'
          ? { state: 'connecting' }
          : { state: 'error', code: 'CONNECT_FAILED', message: 'offline' },
    },
  );

  assert.deepEqual(getRefreshNodeIds(cards), ['node-3']);
});