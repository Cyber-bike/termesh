import * as assert from 'node:assert/strict';
import test from 'node:test';

import {
  SessionRegistry,
  transitionMode,
  transitionSession,
  type Mode,
  type SessionState,
} from './sessionRegistry.ts';

test('mode walks LocalMode to DeviceList and back', () => {
  let mode: Mode = 'LocalMode';
  mode = transitionMode(mode, { type: 'switchToRemote' });
  assert.equal(mode, 'RemoteIdle');
  mode = transitionMode(mode, { type: 'devicesLoaded' });
  assert.equal(mode, 'DeviceList');
  mode = transitionMode(mode, { type: 'switchToLocal' });
  assert.equal(mode, 'LocalMode');
});

test('every mode can return to local', () => {
  for (const mode of ['RemoteIdle', 'DeviceList'] as Mode[]) {
    assert.equal(transitionMode(mode, { type: 'switchToLocal' }), 'LocalMode');
  }
});

test('session happy path: connect, transfer, close', () => {
  let state: SessionState | null = 'Connecting';
  state = transitionSession(state, { type: 'opened' });
  assert.equal(state, 'Connected');
  state = transitionSession(state, { type: 'dropNote' });
  assert.equal(state, 'Transferring');
  state = transitionSession(state, { type: 'transferFinished' });
  assert.equal(state, 'Connected');
});

test('a failed open lands in Error and can be retried', () => {
  assert.equal(transitionSession('Connecting', { type: 'openFailed' }), 'Error');
  assert.equal(transitionSession('Error', { type: 'reconnect' }), 'Connecting');
});

test('losing the connection during a transfer lands in Error', () => {
  assert.equal(transitionSession('Transferring', { type: 'connectionLost' }), 'Error');
});

test('unrelated events leave the session state untouched', () => {
  assert.equal(transitionSession('Connecting', { type: 'dropNote' }), 'Connecting');
  assert.equal(transitionSession('Connected', { type: 'reconnect' }), 'Connected');
});

test('two sessions on the same device are independent', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  registry.open('device-a', 'session-2');

  registry.apply('device-a', 'session-1', { type: 'opened' });
  registry.apply('device-a', 'session-2', { type: 'opened' });
  registry.apply('device-a', 'session-1', { type: 'connectionLost' });

  assert.equal(registry.get('device-a', 'session-1'), 'Error');
  assert.equal(registry.get('device-a', 'session-2'), 'Connected', 'unaffected sibling session');
  assert.equal(registry.countForDevice('device-a'), 2);
});

test('sessions on different devices are independent', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  registry.open('device-b', 'session-1');

  registry.apply('device-a', 'session-1', { type: 'opened' });
  registry.apply('device-a', 'session-1', { type: 'connectionLost' });

  assert.equal(registry.get('device-a', 'session-1'), 'Error');
  assert.equal(registry.get('device-b', 'session-1'), 'Connecting');
});

test('closing a session removes it without touching others', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  registry.open('device-a', 'session-2');
  registry.apply('device-a', 'session-1', { type: 'opened' });
  registry.apply('device-a', 'session-2', { type: 'opened' });

  registry.close('device-a', 'session-1');

  assert.equal(registry.get('device-a', 'session-1'), undefined);
  assert.equal(registry.get('device-a', 'session-2'), 'Connected');
  assert.equal(registry.countForDevice('device-a'), 1);
});

test('closing a session works from any state, including Error', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  registry.apply('device-a', 'session-1', { type: 'openFailed' });
  assert.equal(registry.get('device-a', 'session-1'), 'Error');

  registry.close('device-a', 'session-1');
  assert.equal(registry.get('device-a', 'session-1'), undefined);
});

test('applying an event to an unknown session is a harmless no-op', () => {
  const registry = new SessionRegistry();
  const result = registry.apply('ghost-device', 'ghost-session', { type: 'opened' });
  assert.equal(result, undefined);
  assert.equal(registry.count(), 0);
});

test('opening a duplicate (deviceId, sessionId) pair throws', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  assert.throws(() => registry.open('device-a', 'session-1'));
});

test('forDevice lists only that device\'s sessions, in insertion order', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  registry.open('device-b', 'session-1');
  registry.open('device-a', 'session-2');

  const sessions = registry.forDevice('device-a');
  assert.deepEqual(
    sessions.map((s) => s.sessionId),
    ['session-1', 'session-2']
  );
});

test('closing every session on one device does not affect another device', () => {
  const registry = new SessionRegistry();
  registry.open('device-a', 'session-1');
  registry.open('device-b', 'session-1');

  registry.close('device-a', 'session-1');

  assert.equal(registry.countForDevice('device-a'), 0);
  assert.equal(registry.countForDevice('device-b'), 1);
});
