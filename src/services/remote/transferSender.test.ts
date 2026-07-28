import * as assert from 'node:assert/strict';
import test from 'node:test';

import { decodeFrame, KIND_FILE_CHUNK } from './frameCodec.ts';
import type { CollectedFile } from './noteCollector.ts';
import { TransferSender, type ControlChannel } from './transferSender.ts';

const DEVICE = '3d594650-3436-4c7a-9a15-9b5c3f0f4a11';
const TRANSFER = 'e2f3a4b5-c6d7-4e8f-9a0b-1c2d3e4f5a6b';

interface Recorder extends ControlChannel {
  json: Array<Record<string, unknown>>;
  binary: Uint8Array[];
}

function recorder(): Recorder {
  const json: Array<Record<string, unknown>> = [];
  const binary: Uint8Array[] = [];
  return {
    json,
    binary,
    sendJson: (message) => json.push(message as Record<string, unknown>),
    sendBinary: (frame) => binary.push(frame),
  };
}

function files(...specs: Array<[string, number]>): CollectedFile[] {
  return specs.map(([relativePath, size], index) => ({ index, relativePath, size }));
}

test('a note and its attachment are sent in manifest order', async () => {
  const channel = recorder();
  const contents: Record<string, Uint8Array> = {
    'notes/demo.md': new TextEncoder().encode('# Demo'),
    'assets/a.png': new Uint8Array(1000).fill(7),
  };

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['notes/demo.md', 6], ['assets/a.png', 1000]),
    async (path) => contents[path],
    4 * 1024 * 1024
  );

  const run = sender.run();
  await new Promise((resolve) => setImmediate(resolve));
  sender.complete({ success: true, code: null, message: '' });

  const outcome = await run;
  assert.equal(outcome.success, true);

  const frames = channel.binary.map(decodeFrame);
  assert.deepEqual(frames.map((f) => f.fileIndex), [0, 1]);
  assert.ok(frames.every((f) => f.kind === KIND_FILE_CHUNK));
  assert.ok(frames.every((f) => f.streamId === TRANSFER));

  const types = channel.json.map((m) => m.type);
  assert.deepEqual(types, ['transfer.fileEnd', 'transfer.fileEnd', 'transfer.complete']);
});

test('an empty file sends no chunk but still sends fileEnd', async () => {
  const channel = recorder();

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['empty.md', 0]),
    async () => new Uint8Array(0),
    1024
  );

  const run = sender.run();
  await new Promise((resolve) => setImmediate(resolve));
  sender.complete({ success: true, code: null, message: '' });
  await run;

  assert.equal(channel.binary.length, 0, 'an empty file produces no chunks');
  const fileEnd = channel.json.find((m) => m.type === 'transfer.fileEnd');
  assert.ok(fileEnd);
  assert.equal((fileEnd?.payload as { sentSize: number }).sentSize, 0);
});

test('chunks are paced by the credit window', async () => {
  const channel = recorder();
  const big = new Uint8Array(1024 * 1024).fill(3);

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['big.bin', big.length]),
    async () => big,
    256 * 1024 // room for exactly one chunk
  );

  const run = sender.run();
  await new Promise((resolve) => setImmediate(resolve));

  assert.equal(channel.binary.length, 1, 'the sender must stop at the credit ceiling');

  sender.grantCredit(1024 * 1024);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(channel.binary.length, 4, 'more credit lets the rest through');

  sender.complete({ success: true, code: null, message: '' });
  await run;
});

test('offsets are contiguous within a file', async () => {
  const channel = recorder();
  const data = new Uint8Array(600 * 1024).fill(1);

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['big.bin', data.length]),
    async () => data,
    10 * 1024 * 1024
  );

  const run = sender.run();
  await new Promise((resolve) => setImmediate(resolve));
  sender.complete({ success: true, code: null, message: '' });
  await run;

  let expected = 0;
  for (const frame of channel.binary.map(decodeFrame)) {
    assert.equal(frame.offset, expected, 'the agent rejects a gap outright');
    expected += frame.payload.length;
  }
  assert.equal(expected, data.length);
});

test('a read failure aborts the transfer instead of hanging', async () => {
  const channel = recorder();

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['gone.md', 10]),
    async () => {
      throw new Error('file vanished');
    },
    1024
  );

  const outcome = await sender.run();
  assert.equal(outcome.success, false);
  assert.match(outcome.message, /file vanished/);
  assert.ok(
    channel.json.some((m) => m.type === 'transfer.abort'),
    'the agent must be told to stop rather than wait for its idle timeout'
  );
});

test('a failing result releases a sender blocked on credit', async () => {
  const channel = recorder();
  const big = new Uint8Array(1024 * 1024);

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['big.bin', big.length]),
    async () => big,
    256 * 1024
  );

  const run = sender.run();
  await new Promise((resolve) => setImmediate(resolve));

  // The agent gives up while the sender is waiting for more credit.
  sender.complete({ success: false, code: 'WRITE_FAILED', message: 'receive root is read-only' });

  const outcome = await run;
  assert.equal(outcome.success, false);
  assert.equal(outcome.code, 'WRITE_FAILED');
});

test('progress is reported as bytes land', async () => {
  const channel = recorder();
  const data = new Uint8Array(600 * 1024);
  const seen: number[] = [];

  const sender = new TransferSender(
    channel,
    DEVICE,
    TRANSFER,
    files(['big.bin', data.length]),
    async () => data,
    10 * 1024 * 1024,
    { onProgress: (sent) => seen.push(sent) }
  );

  const run = sender.run();
  await new Promise((resolve) => setImmediate(resolve));
  sender.complete({ success: true, code: null, message: '' });
  await run;

  assert.ok(seen.length >= 3);
  assert.equal(seen[seen.length - 1], data.length);
  assert.deepEqual(seen, [...seen].sort((a, b) => a - b), 'progress must be monotonic');
});
