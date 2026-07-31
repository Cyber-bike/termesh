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
    payload: { event: 'command_end', cwd: '/home/user/project', exitCode: 0 },
  });
  roundtrip({ kind: 'shellEvent', payload: { event: 'prompt_start', cwd: null, exitCode: null } });
  roundtrip({ kind: 'close', payload: { reason: 'peer disconnected' } });
  roundtrip({ kind: 'close', payload: { reason: null } });
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
    payload: { event: 'command_end', cwd: '/tmp/some/fairly/long/path/for/varint/coverage', exitCode: 1 },
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
