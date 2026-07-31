import * as assert from 'node:assert/strict';
import test from 'node:test';

import { byteStreamFromBi, terminalStreamFactory, type IrohBiStream } from './irohStreams.ts';

function fakeBi() {
  const written: number[][] = [];
  let finished = 0;
  const reads: (number[] | null)[] = [];
  const bi: IrohBiStream = {
    send: {
      async writeAll(bytes) {
        written.push([...bytes]);
      },
      async finish() {
        finished += 1;
      },
    },
    recv: {
      async read() {
        return reads.shift() ?? null;
      },
    },
  };
  return { bi, written, reads, finished: () => finished };
}

test('write converts Uint8Array to the number[] the binding expects', async () => {
  const { bi, written } = fakeBi();
  const stream = byteStreamFromBi(bi);

  await stream.write(new Uint8Array([0, 127, 255]));
  assert.deepEqual(written, [[0, 127, 255]]);
});

test('read converts chunks back and maps empty/absent chunks to end-of-stream', async () => {
  const { bi, reads } = fakeBi();
  reads.push([1, 2, 3], [], null);
  const stream = byteStreamFromBi(bi);

  assert.deepEqual(await stream.read(), new Uint8Array([1, 2, 3]));
  assert.equal(await stream.read(), null, 'empty chunk is end-of-stream');
  assert.equal(await stream.read(), null, 'null chunk is end-of-stream');
});

test('finishWrite finishes the send half and swallows late failures', () => {
  const { bi, finished } = fakeBi();
  const stream = byteStreamFromBi(bi);

  stream.finishWrite();
  assert.equal(finished(), 1);

  const failing = byteStreamFromBi({
    ...bi,
    send: {
      async writeAll() {},
      async finish() {
        throw new Error('connection already closed');
      },
    },
  });
  failing.finishWrite(); // must not produce an unhandled rejection
});

test('terminalStreamFactory opens a fresh bi-stream per call', async () => {
  let opened = 0;
  const factory = terminalStreamFactory({
    async openBi() {
      opened += 1;
      return fakeBi().bi;
    },
    close() {},
    async closed() {
      return '';
    },
  });

  await factory();
  await factory();
  assert.equal(opened, 2, 'each session needs its own stream (doc 8.2)');
});
