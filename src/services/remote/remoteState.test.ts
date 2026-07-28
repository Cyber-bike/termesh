import * as assert from 'node:assert/strict';
import test from 'node:test';

import {
  capabilities,
  reachableStates,
  transition,
  type RemoteState,
} from './remoteState.ts';

test('the happy path walks LocalMode to Connected and back', () => {
  let state: RemoteState = 'LocalMode';
  state = transition(state, { type: 'switchToRemote' });
  assert.equal(state, 'RemoteIdle');
  state = transition(state, { type: 'connect' });
  assert.equal(state, 'Connecting');
  state = transition(state, { type: 'opened' });
  assert.equal(state, 'Connected');
  state = transition(state, { type: 'disconnect' });
  assert.equal(state, 'RemoteIdle');
});

test('a transfer returns to Connected', () => {
  let state: RemoteState = 'Connected';
  state = transition(state, { type: 'dropNote' });
  assert.equal(state, 'Transferring');
  state = transition(state, { type: 'transferFinished' });
  assert.equal(state, 'Connected');
});

test('losing the connection during a transfer lands in Error', () => {
  // The edge doc 5.3 originally omitted.
  assert.equal(transition('Transferring', { type: 'connectionLost' }), 'Error');
});

test('Error can be left three ways', () => {
  assert.equal(transition('Error', { type: 'connect' }), 'Connecting');
  assert.equal(transition('Error', { type: 'chooseDevice' }), 'RemoteIdle');
  assert.equal(transition('Error', { type: 'switchToLocal' }), 'LocalMode');
});

test('every remote state can return to local mode', () => {
  for (const state of ['RemoteIdle', 'Connected', 'Transferring', 'Error'] as RemoteState[]) {
    assert.equal(
      transition(state, { type: 'switchToLocal' }),
      'LocalMode',
      `${state} must be able to go back to the local terminal`
    );
  }
});

test('unrelated events leave the state untouched', () => {
  assert.equal(transition('LocalMode', { type: 'opened' }), 'LocalMode');
  assert.equal(transition('Connecting', { type: 'dropNote' }), 'Connecting');
  assert.equal(transition('RemoteIdle', { type: 'transferFinished' }), 'RemoteIdle');
});

test('no state is unreachable', () => {
  const reachable = reachableStates();
  for (const state of [
    'LocalMode',
    'RemoteIdle',
    'Connecting',
    'Connected',
    'Transferring',
    'Error',
  ] as RemoteState[]) {
    assert.ok(reachable.has(state), `${state} is unreachable from LocalMode`);
  }
});

test('capabilities match the table in doc 5.3', () => {
  assert.deepEqual(capabilities('LocalMode'), { input: true, drop: true, deviceSelection: false });
  assert.deepEqual(capabilities('RemoteIdle'), { input: false, drop: false, deviceSelection: true });
  assert.deepEqual(capabilities('Connecting'), { input: false, drop: false, deviceSelection: false });
  assert.deepEqual(capabilities('Connected'), { input: true, drop: true, deviceSelection: false });
  assert.deepEqual(capabilities('Transferring'), { input: true, drop: false, deviceSelection: false });
  assert.deepEqual(capabilities('Error'), { input: false, drop: false, deviceSelection: true });
});

test('a second note cannot be dropped while one is transferring', () => {
  assert.equal(capabilities('Transferring').drop, false);
  assert.equal(transition('Transferring', { type: 'dropNote' }), 'Transferring');
});
