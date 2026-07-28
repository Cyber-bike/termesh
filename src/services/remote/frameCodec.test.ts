import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as path from 'node:path';
import test from 'node:test';

import {
  decodeFrame,
  encodeFrame,
  FILE_PAYLOAD_MAX,
  FrameError,
  HEADER_BYTES,
  KIND_FILE_CHUNK,
  KIND_TERMINAL_INPUT,
  KIND_TERMINAL_OUTPUT,
  MESSAGE_MAX,
  TERMINAL_FILE_INDEX,
  TERMINAL_PAYLOAD_MAX,
} from './frameCodec.ts';

const SESSION = 'c9e0f1a2-b3c4-4d5e-9f60-718293a4b5c6';
const TRANSFER = 'e2f3a4b5-c6d7-4e8f-9a0b-1c2d3e4f5a6b';

const framesDir = path.join(process.cwd(), 'protocol', 'fixtures', 'frames');

function loadVector(name: string): Uint8Array {
  const raw = fs.readFileSync(path.join(framesDir, `${name}.hex`), 'utf8');
  const hex = raw
    .split('\n')
    .filter((line) => !line.trimStart().startsWith('#'))
    .join('')
    .trim();
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i += 1) {
    bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

test('a terminal frame round trips', () => {
  const payload = new TextEncoder().encode('hello\r\n');
  const encoded = encodeFrame({
    kind: KIND_TERMINAL_OUTPUT,
    streamId: SESSION,
    fileIndex: TERMINAL_FILE_INDEX,
    offset: 0,
    payload,
  });

  assert.equal(encoded.length, HEADER_BYTES + payload.length);

  const decoded = decodeFrame(encoded);
  assert.equal(decoded.kind, KIND_TERMINAL_OUTPUT);
  assert.equal(decoded.streamId, SESSION);
  assert.equal(decoded.fileIndex, TERMINAL_FILE_INDEX);
  assert.equal(decoded.offset, 0);
  assert.deepEqual(Array.from(decoded.payload), Array.from(payload));
});

test('a file chunk survives an offset above 2^32', () => {
  const encoded = encodeFrame({
    kind: KIND_FILE_CHUNK,
    streamId: TRANSFER,
    fileIndex: 7,
    offset: 4294967296,
    payload: new Uint8Array(1024).fill(0xab),
  });

  const decoded = decodeFrame(encoded);
  assert.equal(decoded.fileIndex, 7);
  assert.equal(decoded.offset, 4294967296);
  assert.equal(decoded.payload.length, 1024);
});

test('the largest file chunk matches the documented message ceiling', () => {
  const encoded = encodeFrame({
    kind: KIND_FILE_CHUNK,
    streamId: TRANSFER,
    fileIndex: 0,
    offset: 0,
    payload: new Uint8Array(FILE_PAYLOAD_MAX),
  });
  assert.equal(encoded.length, 262182, 'doc 8.5 fixes the maximum binary message');
  assert.equal(encoded.length, MESSAGE_MAX);
});

test('malformed headers are rejected', () => {
  const good = encodeFrame({
    kind: KIND_TERMINAL_OUTPUT,
    streamId: SESSION,
    fileIndex: TERMINAL_FILE_INDEX,
    offset: 0,
    payload: new TextEncoder().encode('abc'),
  });

  const mutate = (mutation: (b: Uint8Array) => void): Uint8Array => {
    const copy = good.slice();
    mutation(copy);
    return copy;
  };

  const cases: Array<[string, (b: Uint8Array) => void, RegExp]> = [
    ['bad magic', (b) => { b[0] = 0x58; }, /bad magic/],
    ['bad version', (b) => { b[2] = 0x02; }, /unsupported frame version/],
    ['unknown kind', (b) => { b[3] = 0x09; }, /unknown kind/],
    ['non-zero flags', (b) => { b[4] = 1; }, /flags must be 0/],
    ['non-zero reserved', (b) => { b[5] = 1; }, /reserved byte must be 0/],
    ['length mismatch', (b) => { new DataView(b.buffer).setUint32(6, 999, false); }, /does not match/],
    ['terminal frame with a file index', (b) => { new DataView(b.buffer).setUint32(26, 3, false); }, /fileIndex/],
  ];

  for (const [name, mutation, expected] of cases) {
    assert.throws(() => decodeFrame(mutate(mutation)), expected, name);
  }

  assert.throws(() => decodeFrame(good.subarray(0, 20)), /truncated header/);
});

test('an oversized terminal payload is refused at encode time', () => {
  assert.throws(
    () =>
      encodeFrame({
        kind: KIND_TERMINAL_OUTPUT,
        streamId: SESSION,
        fileIndex: TERMINAL_FILE_INDEX,
        offset: 0,
        payload: new Uint8Array(TERMINAL_PAYLOAD_MAX + 1),
      }),
    FrameError
  );
});

test('an empty payload is legal', () => {
  const encoded = encodeFrame({
    kind: KIND_TERMINAL_INPUT,
    streamId: SESSION,
    fileIndex: TERMINAL_FILE_INDEX,
    offset: 0,
    payload: new Uint8Array(0),
  });
  assert.equal(encoded.length, HEADER_BYTES);
  assert.equal(decodeFrame(encoded).payload.length, 0);
});

test('the plugin agrees with the shared frame vectors', () => {
  const index = JSON.parse(fs.readFileSync(path.join(framesDir, 'index.json'), 'utf8')) as Array<{
    name: string;
    expect: 'accept' | 'reject';
    note: string;
  }>;

  assert.ok(index.length > 0, 'no shared vectors found');

  for (const vector of index) {
    const bytes = loadVector(vector.name);
    if (vector.expect === 'accept') {
      assert.doesNotThrow(() => decodeFrame(bytes), `${vector.name} (${vector.note})`);
    } else {
      assert.throws(() => decodeFrame(bytes), `${vector.name} should be rejected (${vector.note})`);
    }
  }
});

test('re-encoding a shared vector reproduces its bytes', () => {
  for (const name of ['valid-terminal-output', 'valid-file-chunk', 'valid-empty-payload']) {
    const bytes = loadVector(name);
    const reencoded = encodeFrame(decodeFrame(bytes));
    assert.deepEqual(Array.from(reencoded), Array.from(bytes), name);
  }
});
