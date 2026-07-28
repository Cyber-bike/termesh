import * as assert from 'node:assert/strict';
import test from 'node:test';

import { chunkify, CreditWindow, FILE_CHUNK_BYTES } from './creditWindow.ts';

test('sending within the window does not block', async () => {
  const window = new CreditWindow(1024);
  await window.reserve(512);
  await window.reserve(512);
  assert.equal(window.sentBytes, 1024);
  assert.equal(window.availableBytes, 0);
});

test('a reservation past the window waits for a grant', async () => {
  const window = new CreditWindow(100);
  await window.reserve(100);

  let released = false;
  const pending = window.reserve(50).then(() => {
    released = true;
  });

  // Nothing should have gone out yet.
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(released, false, 'the sender must wait for credit');

  window.grant(200);
  await pending;
  assert.equal(released, true);
  assert.equal(window.sentBytes, 150);
});

test('grants are cumulative and never shrink the window', () => {
  const window = new CreditWindow(1000);
  window.grant(2000);
  assert.equal(window.grantedBytes, 2000);

  // A replayed or reordered grant must not reduce the ceiling.
  window.grant(1500);
  assert.equal(window.grantedBytes, 2000);
});

test('a failed transfer wakes every waiter instead of hanging', async () => {
  const window = new CreditWindow(10);
  await window.reserve(10);

  const first = window.reserve(5);
  const second = window.reserve(5);

  window.fail(new Error('transfer aborted'));

  await assert.rejects(first, /transfer aborted/);
  await assert.rejects(second, /transfer aborted/);

  // And later reservations fail immediately rather than blocking forever.
  await assert.rejects(window.reserve(1), /transfer aborted/);
});

test('a large file splits into protocol-sized chunks', () => {
  const data = new Uint8Array(FILE_CHUNK_BYTES * 2 + 100);
  const chunks = Array.from(chunkify(data));

  assert.equal(chunks.length, 3);
  assert.deepEqual(chunks.map((c) => c.offset), [0, FILE_CHUNK_BYTES, FILE_CHUNK_BYTES * 2]);
  assert.equal(chunks[0].slice.length, FILE_CHUNK_BYTES);
  assert.equal(chunks[2].slice.length, 100);

  // Offsets must be contiguous; the agent rejects a gap outright.
  let expected = 0;
  for (const chunk of chunks) {
    assert.equal(chunk.offset, expected);
    expected += chunk.slice.length;
  }
  assert.equal(expected, data.length);
});

test('an empty file produces no chunks at all', () => {
  // Doc 10.4: the agent creates it from transfer.fileEnd instead.
  assert.deepEqual(Array.from(chunkify(new Uint8Array(0))), []);
});

test('the window paces a realistic transfer', async () => {
  const window = new CreditWindow(4 * 1024 * 1024);
  const data = new Uint8Array(6 * 1024 * 1024);

  let sent = 0;
  let grantsNeeded = 0;

  for (const { slice } of chunkify(data)) {
    if (window.availableBytes < slice.length) {
      grantsNeeded += 1;
      window.grant(window.grantedBytes + 1024 * 1024);
    }
    await window.reserve(slice.length);
    sent += slice.length;
  }

  assert.equal(sent, data.length);
  assert.ok(grantsNeeded > 0, 'a 6 MiB file must need more credit than the initial window');
});
