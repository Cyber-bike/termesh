import * as assert from 'node:assert/strict';
import test from 'node:test';

import {
  encodeTerminalStreamFrame,
  TerminalStreamFrameDecoder,
  TerminalStreamFrameError,
  type TerminalStreamFrame,
} from './terminalStreamFrame.ts';

function roundtrip(frame: TerminalStreamFrame): void {
  const encoded = encodeTerminalStreamFrame(frame);
  const decoder = new TerminalStreamFrameDecoder();
  decoder.push(encoded);
  assert.deepEqual(decoder.nextFrame(), frame);
  assert.equal(decoder.nextFrame(), null, 'buffer must be drained');
}

test('every frame kind round-trips', () => {
  roundtrip({ kind: 'open', payload: { cols: 80, rows: 24 } });
  roundtrip({ kind: 'opened', payload: { sessionId: 'abc-123', shell: '/bin/bash' } });
  roundtrip({ kind: 'error', payload: { message: 'SESSION_LIMIT_REACHED' } });
  roundtrip({ kind: 'data', payload: new TextEncoder().encode('echo hi\n') });
  roundtrip({ kind: 'data', payload: new Uint8Array(0) });
  roundtrip({ kind: 'resize', payload: { cols: 120, rows: 40 } });
  roundtrip({
    kind: 'shellEvent',
    payload: { event: 'command_end', source: 'osc133', cwd: '/home/user/project', exitCode: 0 },
  });
  roundtrip({
    kind: 'shellEvent',
    payload: { event: 'prompt_start', source: null, cwd: null, exitCode: null },
  });
  roundtrip({ kind: 'close', payload: { reason: 'peer disconnected', exitCode: null } });
  roundtrip({ kind: 'close', payload: { reason: 'shell_exited', exitCode: 0 } });
  roundtrip({ kind: 'close', payload: { reason: null, exitCode: null } });
  roundtrip({ kind: 'fsList', payload: { path: '/home/user/project' } });
  roundtrip({
    kind: 'fsListResult',
    payload: {
      entries: [
        { name: 'src', isDirectory: true },
        { name: 'readme.md', isDirectory: false },
      ],
    },
  });
  roundtrip({ kind: 'fsListResult', payload: { entries: [] } });
  roundtrip({ kind: 'fsChanged', payload: { kind: 'unknown' } });
  roundtrip({
    kind: 'transferManifest',
    payload: {
      transferId: 'transfer-1',
      rootNote: 'notes/demo.md',
      entries: [
        { index: 0, relativePath: 'notes/demo.md', size: 11 },
        { index: 1, relativePath: 'assets/img.png', size: 0 },
      ],
      sessionId: null,
      targetPath: null,
    },
  });
  roundtrip({
    kind: 'transferManifest',
    payload: {
      transferId: 'transfer-2',
      rootNote: 'a.md',
      entries: [{ index: 0, relativePath: 'a.md', size: 1 }],
      sessionId: 'session-abc',
      targetPath: null,
    },
  });
  roundtrip({
    kind: 'transferManifest',
    payload: {
      transferId: 'transfer-3',
      rootNote: 'a.md',
      entries: [{ index: 0, relativePath: 'a.md', size: 1 }],
      sessionId: null,
      targetPath: '/home/user/project/notes',
    },
  });
  roundtrip({ kind: 'transferAccepted', payload: { grantedBytes: 4 * 1024 * 1024 } });
  roundtrip({
    kind: 'transferChunk',
    payload: { fileIndex: 0, offset: 0, data: new TextEncoder().encode('hello world') },
  });
  roundtrip({ kind: 'transferChunk', payload: { fileIndex: 3, offset: 300_000, data: new Uint8Array(0) } });
  roundtrip({ kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 11 } });
  roundtrip({ kind: 'transferCredit', payload: { grantedBytes: 8 * 1024 * 1024 } });
  roundtrip({ kind: 'transferComplete', payload: {} });
  roundtrip({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });
  roundtrip({
    kind: 'transferResult',
    payload: { success: false, code: 'WRITE_FAILED', message: 'disk full' },
  });
  roundtrip({ kind: 'transferPullRequest', payload: { path: '/home/user/project/notes' } });
  roundtrip({
    kind: 'transferPullManifest',
    payload: {
      entries: [
        { index: 0, relativePath: 'notes/demo.md', size: 11 },
        { index: 1, relativePath: 'notes/assets/img.png', size: 0 },
      ],
    },
  });
  roundtrip({ kind: 'transferPullManifest', payload: { entries: [] } });
});

test('a transferChunk with a large offset round-trips its varint', () => {
  roundtrip({
    kind: 'transferChunk',
    payload: { fileIndex: 1, offset: 60 * 1024 * 1024, data: new Uint8Array(64).fill(7) },
  });
});

test('fsListResult with a malformed entry is a protocol error', () => {
  const decoder = new TerminalStreamFrameDecoder();
  // kind 0x08 (fsListResult); entry missing "isDirectory".
  const payload = new TextEncoder().encode(JSON.stringify({ entries: [{ name: 'a.txt' }] }));
  decoder.push(new Uint8Array([0x08, payload.length, ...payload]));
  assert.throws(() => decoder.nextFrame(), TerminalStreamFrameError);
});

test('two frames back to back decode in order', () => {
  const first: TerminalStreamFrame = { kind: 'data', payload: new TextEncoder().encode('one') };
  const second: TerminalStreamFrame = { kind: 'data', payload: new TextEncoder().encode('two') };

  const decoder = new TerminalStreamFrameDecoder();
  decoder.push(encodeTerminalStreamFrame(first));
  decoder.push(encodeTerminalStreamFrame(second));

  assert.deepEqual(decoder.nextFrame(), first);
  assert.deepEqual(decoder.nextFrame(), second);
  assert.equal(decoder.nextFrame(), null);
});

test('a frame split across many single-byte chunks still decodes', () => {
  const frame: TerminalStreamFrame = {
    kind: 'shellEvent',
    payload: {
      event: 'command_end',
      source: 'osc633',
      cwd: '/tmp/some/fairly/long/path/for/varint/coverage',
      exitCode: 1,
    },
  };
  const encoded = encodeTerminalStreamFrame(frame);

  const decoder = new TerminalStreamFrameDecoder();
  for (const byte of encoded) {
    assert.equal(decoder.nextFrame(), null, 'must not decode early');
    decoder.push(new Uint8Array([byte]));
  }
  assert.deepEqual(decoder.nextFrame(), frame);
});

test('an unknown kind byte is a protocol error', () => {
  const decoder = new TerminalStreamFrameDecoder();
  decoder.push(new Uint8Array([0x7f, 0x00])); // kind 0x7f, zero-length payload
  assert.throws(() => decoder.nextFrame(), TerminalStreamFrameError);
});

test('a length prefix over the cap is rejected', () => {
  const decoder = new TerminalStreamFrameDecoder();
  const lenBytes: number[] = [];
  let value = 1024 * 1024 + 1;
  for (;;) {
    const byte = value & 0x7f;
    value >>>= 7;
    if (value === 0) {
      lenBytes.push(byte);
      break;
    }
    lenBytes.push(byte | 0x80);
  }
  decoder.push(new Uint8Array([0x01, ...lenBytes]));
  assert.throws(() => decoder.nextFrame(), TerminalStreamFrameError);
});

test('malformed JSON in a structured frame is a protocol error', () => {
  const decoder = new TerminalStreamFrameDecoder();
  decoder.push(new Uint8Array([0x02, 0x02, ...new TextEncoder().encode('{}')])); // resize missing cols/rows
  assert.throws(() => decoder.nextFrame(), TerminalStreamFrameError);
});

test('non-object JSON in a structured frame is a protocol error', () => {
  const decoder = new TerminalStreamFrameDecoder();
  const payload = new TextEncoder().encode('[1,2]');
  const lenBytes: number[] = [];
  let value = payload.length;
  for (;;) {
    const byte = value & 0x7f;
    value >>>= 7;
    if (value === 0) {
      lenBytes.push(byte);
      break;
    }
    lenBytes.push(byte | 0x80);
  }
  decoder.push(new Uint8Array([0x02, ...lenBytes, ...payload]));
  assert.throws(() => decoder.nextFrame(), TerminalStreamFrameError);
});

test('feeding an empty chunk is a harmless no-op', () => {
  const decoder = new TerminalStreamFrameDecoder();
  decoder.push(new Uint8Array(0));
  assert.equal(decoder.nextFrame(), null);
});
