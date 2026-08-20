'use strict';

/**
 * Frame-level contract tests. Every case is also written to fixtures/frames/ as
 * a hex vector with an expectation, so the Rust decoder can be held to the same
 * accept/reject results (doc 8.1).
 */

const fs = require('fs');
const path = require('path');
const assert = require('assert');
const codec = require('./frame-codec');

const FRAME_DIR = path.join(__dirname, '..', 'fixtures', 'frames');
const SESSION = 'c9e0f1a2-b3c4-4d5e-9f60-718293a4b5c6';
const TRANSFER = 'e2f3a4b5-c6d7-4e8f-9a0b-1c2d3e4f5a6b';

const vectors = [];
const record = (name, buf, expect, note) => {
  vectors.push({ name, hex: buf.toString('hex'), expect, note });
  return buf;
};

let failures = 0;
const check = (name, fn) => {
  try {
    fn();
    console.log(`  ok  ${name}`);
  } catch (err) {
    console.error(`  FAIL ${name}: ${err.message}`);
    failures += 1;
  }
};

// --- round trip -------------------------------------------------------------

check('terminal output round trip', () => {
  const payload = Buffer.from('hello\r\n', 'utf8');
  const buf = record(
    'valid-terminal-output',
    codec.encode({ kind: codec.KIND_TERMINAL_OUTPUT, streamId: SESSION, offset: 0, payload }),
    'accept',
    'kind=0x02, fileIndex must be 0xFFFFFFFF'
  );
  assert.strictEqual(buf.length, codec.HEADER_BYTES + payload.length);
  const out = codec.decode(buf);
  assert.strictEqual(out.kind, codec.KIND_TERMINAL_OUTPUT);
  assert.strictEqual(out.streamId, SESSION);
  assert.strictEqual(out.fileIndex, codec.TERMINAL_FILE_INDEX);
  assert.strictEqual(out.offset, 0n);
  assert.deepStrictEqual(out.payload, payload);
});

check('file chunk round trip at a large offset', () => {
  const payload = Buffer.alloc(1024, 0xab);
  const buf = record(
    'valid-file-chunk',
    codec.encode({ kind: codec.KIND_FILE_CHUNK, streamId: TRANSFER, fileIndex: 7, offset: 4294967296, payload }),
    'accept',
    'offset above 2^32 exercises the u64 field'
  );
  const out = codec.decode(buf);
  assert.strictEqual(out.fileIndex, 7);
  assert.strictEqual(out.offset, 4294967296n);
  assert.strictEqual(out.payload.length, 1024);
});

check('empty payload is legal', () => {
  const buf = record(
    'valid-empty-payload',
    codec.encode({ kind: codec.KIND_TERMINAL_INPUT, streamId: SESSION, offset: 0, payload: Buffer.alloc(0) }),
    'accept',
    'zero-length payload, header only'
  );
  assert.strictEqual(buf.length, codec.HEADER_BYTES);
  assert.strictEqual(codec.decode(buf).payload.length, 0);
});

check('max sized file chunk equals the documented message ceiling', () => {
  const payload = Buffer.alloc(codec.FILE_PAYLOAD_MAX, 0x5a);
  const buf = codec.encode({ kind: codec.KIND_FILE_CHUNK, streamId: TRANSFER, fileIndex: 0, offset: 0, payload });
  assert.strictEqual(buf.length, 262182, 'doc 8.5 states the max Binary message is 262182 bytes');
  assert.strictEqual(buf.length, codec.MESSAGE_MAX);
});

// --- rejection --------------------------------------------------------------

const mutate = (fn) => {
  const buf = codec.encode({
    kind: codec.KIND_TERMINAL_OUTPUT,
    streamId: SESSION,
    offset: 0,
    payload: Buffer.from('abc', 'utf8')
  });
  fn(buf);
  return buf;
};

const rejects = [
  ['bad-magic', 'bad magic', (b) => { b[0] = 0x58; }],
  ['bad-version', 'unsupported frame version', (b) => { b[2] = 0x02; }],
  ['unknown-kind', 'unknown kind', (b) => { b[3] = 0x09; }],
  ['nonzero-flags', 'flags must be 0', (b) => { b[4] = 0x01; }],
  ['nonzero-reserved', 'reserved byte must be 0', (b) => { b[5] = 0x01; }],
  ['length-mismatch', 'does not match', (b) => { b.writeUInt32BE(999, 6); }],
  ['terminal-with-file-index', 'terminal frames must set fileIndex', (b) => { b.writeUInt32BE(3, 26); }]
];

for (const [name, expectedMessage, mutation] of rejects) {
  check(`rejects ${name}`, () => {
    const buf = record(`invalid-${name}`, mutate(mutation), 'reject', expectedMessage);
    assert.throws(() => codec.decode(buf), (err) => {
      assert.strictEqual(err.code, 'PROTOCOL_ERROR');
      assert.ok(err.message.includes(expectedMessage), `got "${err.message}"`);
      return true;
    });
  });
}

check('rejects a truncated header', () => {
  const full = codec.encode({ kind: codec.KIND_TERMINAL_OUTPUT, streamId: SESSION, offset: 0, payload: Buffer.from('abc') });
  const buf = record('invalid-truncated-header', full.subarray(0, 20), 'reject', 'truncated header');
  assert.throws(() => codec.decode(buf), /truncated header/);
});

check('accepts a file chunk with fileIndex above the old 256-file cap', () => {
  const payload = Buffer.from('x');
  const buf = codec.encode({ kind: codec.KIND_FILE_CHUNK, streamId: TRANSFER, fileIndex: 0, offset: 0, payload });
  buf.writeUInt32BE(300, 26);
  record('valid-file-index-above-legacy-cap', buf, 'accept', 'file count is unbounded; only the reserved sentinel is excluded');
  assert.strictEqual(codec.decode(buf).fileIndex, 300);
});

check('rejects a file chunk whose fileIndex is the reserved terminal sentinel', () => {
  const payload = Buffer.from('x');
  const buf = codec.encode({ kind: codec.KIND_FILE_CHUNK, streamId: TRANSFER, fileIndex: 0, offset: 0, payload });
  buf.writeUInt32BE(0xffffffff, 26);
  record('invalid-file-index-reserved-sentinel', buf, 'reject', 'fileIndex 0xFFFFFFFF is reserved for terminal frames');
  assert.throws(() => codec.decode(buf), /reserved for terminal frames/);
});

check('rejects an oversized terminal payload at encode time', () => {
  assert.throws(
    () => codec.encode({
      kind: codec.KIND_TERMINAL_OUTPUT,
      streamId: SESSION,
      offset: 0,
      payload: Buffer.alloc(codec.TERMINAL_PAYLOAD_MAX + 1)
    }),
    /exceeds the limit/
  );
});

// --- offset counting domains ------------------------------------------------

check('terminal input and output offsets are counted separately', () => {
  const tracker = new codec.OffsetTracker();
  const mk = (kind, offset, len) =>
    codec.decode(codec.encode({ kind, streamId: SESSION, offset, payload: Buffer.alloc(len, 1) }));

  assert.strictEqual(tracker.accept(mk(codec.KIND_TERMINAL_OUTPUT, 0, 100)).ok, true);
  // Input starts at 0 even though output already consumed 100 bytes.
  assert.strictEqual(tracker.accept(mk(codec.KIND_TERMINAL_INPUT, 0, 10)).ok, true);
  assert.strictEqual(tracker.accept(mk(codec.KIND_TERMINAL_OUTPUT, 100, 50)).ok, true);
  assert.strictEqual(tracker.accept(mk(codec.KIND_TERMINAL_INPUT, 10, 5)).ok, true);
});

check('file offsets are counted per fileIndex', () => {
  const tracker = new codec.OffsetTracker();
  const mk = (fileIndex, offset, len) =>
    codec.decode(codec.encode({ kind: codec.KIND_FILE_CHUNK, streamId: TRANSFER, fileIndex, offset, payload: Buffer.alloc(len, 2) }));

  assert.strictEqual(tracker.accept(mk(0, 0, 256)).ok, true);
  // A second file in the same transfer restarts at 0.
  assert.strictEqual(tracker.accept(mk(1, 0, 128)).ok, true);
  assert.strictEqual(tracker.accept(mk(0, 256, 256)).ok, true);
});

check('a file offset gap is fatal, a terminal offset gap is not', () => {
  const tracker = new codec.OffsetTracker();
  const fileGap = codec.decode(codec.encode({ kind: codec.KIND_FILE_CHUNK, streamId: TRANSFER, fileIndex: 0, offset: 64, payload: Buffer.alloc(8) }));
  const r1 = tracker.accept(fileGap);
  assert.strictEqual(r1.ok, false);
  assert.strictEqual(r1.fatal, true);

  const termGap = codec.decode(codec.encode({ kind: codec.KIND_TERMINAL_OUTPUT, streamId: SESSION, offset: 64, payload: Buffer.alloc(8) }));
  const r2 = tracker.accept(termGap);
  assert.strictEqual(r2.ok, false);
  assert.strictEqual(r2.fatal, false);
});

// --- emit shared vectors ----------------------------------------------------

fs.mkdirSync(FRAME_DIR, { recursive: true });
for (const v of vectors) {
  const body = `# ${v.note}\n# expect: ${v.expect}\n${v.hex}\n`;
  fs.writeFileSync(path.join(FRAME_DIR, `${v.name}.hex`), body);
}
fs.writeFileSync(
  path.join(FRAME_DIR, 'index.json'),
  JSON.stringify(vectors.map(({ name, expect, note }) => ({ name, expect, note })), null, 2) + '\n'
);
console.log(`\nWrote ${vectors.length} frame vectors to fixtures/frames/`);

if (failures > 0) {
  console.error(`${failures} failure(s)`);
  process.exit(1);
}
console.log('All frame codec tests passed');
