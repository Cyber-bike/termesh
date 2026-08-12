import * as assert from 'node:assert/strict';
import test from 'node:test';

import type { CollectedFile } from './noteCollector.ts';
import { TransferStreamSender } from './transferStreamSender.ts';
import type { ByteStream } from './terminalStreamTransport.ts';
import {
  encodeTerminalStreamFrame,
  TerminalStreamFrameDecoder,
  type TerminalStreamFrame,
} from './terminalStreamFrame.ts';

/** Single-consumer async queue backing one direction of a stream pair (mirrors terminalStreamTransport.test.ts). */
class Queue {
  private items: (Uint8Array | null)[] = [];
  private waiter: ((item: Uint8Array | null) => void) | null = null;

  push(item: Uint8Array | null): void {
    if (this.waiter) {
      const waiter = this.waiter;
      this.waiter = null;
      waiter(item);
      return;
    }
    this.items.push(item);
  }

  async pop(): Promise<Uint8Array | null> {
    if (this.items.length > 0) return this.items.shift()!;
    return new Promise((resolve) => {
      this.waiter = resolve;
    });
  }
}

function streamPair(): [ByteStream, ByteStream] {
  const aToB = new Queue();
  const bToA = new Queue();
  const make = (outgoing: Queue, incoming: Queue): ByteStream => ({
    async write(bytes) {
      outgoing.push(bytes.slice());
    },
    async read() {
      return incoming.pop();
    },
    finishWrite() {
      outgoing.push(null);
    },
  });
  return [make(aToB, bToA), make(bToA, aToB)];
}

/** The agent's end of one transfer stream, driven frame-by-frame by each test. */
class FakeAgent {
  private readonly decoder = new TerminalStreamFrameDecoder();

  constructor(private readonly stream: ByteStream) {}

  async send(frame: TerminalStreamFrame): Promise<void> {
    await this.stream.write(encodeTerminalStreamFrame(frame));
  }

  async nextFrame(): Promise<TerminalStreamFrame> {
    for (;;) {
      const frame = this.decoder.nextFrame();
      if (frame) return frame;
      const chunk = await this.stream.read();
      assert.ok(chunk !== null, 'the sender ended the stream unexpectedly');
      this.decoder.push(chunk);
    }
  }
}

function file(index: number, relativePath: string, size: number): CollectedFile {
  return { index, relativePath, size };
}

function setup(files: CollectedFile[], readFile: (path: string) => Promise<Uint8Array>) {
  const [clientEnd, agentEnd] = streamPair();
  const agent = new FakeAgent(agentEnd);
  const sender = new TransferStreamSender(async () => clientEnd, 'transfer-1', files, readFile);
  return { sender, agent };
}

test('a full transfer sends the manifest, chunks, fileEnd and complete, then resolves on success', async () => {
  const files = [file(0, 'notes/demo.md', 11)];
  const { sender, agent } = setup(files, async () => new TextEncoder().encode('hello world'));

  const run = sender.run();

  assert.deepEqual(await agent.nextFrame(), {
    kind: 'transferManifest',
    payload: { transferId: 'transfer-1', rootNote: 'notes/demo.md', entries: [{ index: 0, relativePath: 'notes/demo.md', size: 11 }], sessionId: null },
  });
  await agent.send({ kind: 'transferAccepted', payload: { grantedBytes: 4 * 1024 * 1024 } });

  const chunk = await agent.nextFrame();
  assert.equal(chunk.kind, 'transferChunk');
  assert.deepEqual(
    chunk.kind === 'transferChunk' ? new TextDecoder().decode(chunk.payload.data) : null,
    'hello world',
  );

  const fileEnd = await agent.nextFrame();
  assert.deepEqual(fileEnd, { kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 11 } });

  const complete = await agent.nextFrame();
  assert.deepEqual(complete, { kind: 'transferComplete', payload: {} });

  await agent.send({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });

  const outcome = await run;
  assert.deepEqual(outcome, { success: true, code: null, message: '' });
});

test('an empty file still sends fileEnd, never a chunk', async () => {
  const files = [file(0, 'empty.md', 0)];
  const { sender, agent } = setup(files, async () => new Uint8Array(0));

  const run = sender.run();
  await agent.nextFrame(); // manifest
  await agent.send({ kind: 'transferAccepted', payload: { grantedBytes: 4 * 1024 * 1024 } });

  const next = await agent.nextFrame();
  assert.deepEqual(next, { kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 0 } });

  await agent.nextFrame(); // complete
  await agent.send({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });
  assert.deepEqual(await run, { success: true, code: null, message: '' });
});

test('a rejected manifest resolves with the agent-provided error, parsed code', async () => {
  const files = [file(0, 'a.md', 1)];
  const { sender, agent } = setup(files, async () => new TextEncoder().encode('a'));

  const run = sender.run();
  await agent.nextFrame();
  await agent.send({ kind: 'error', payload: { message: 'TRANSFER_REJECTED: bad path' } });

  assert.deepEqual(await run, { success: false, code: 'TRANSFER_REJECTED', message: 'TRANSFER_REJECTED: bad path' });
});

test('a failure result mid-transfer resolves with success: false', async () => {
  const files = [file(0, 'a.md', 1)];
  const { sender, agent } = setup(files, async () => new TextEncoder().encode('a'));

  const run = sender.run();
  await agent.nextFrame(); // manifest
  await agent.send({ kind: 'transferAccepted', payload: { grantedBytes: 4 * 1024 * 1024 } });
  await agent.nextFrame(); // chunk
  await agent.nextFrame(); // fileEnd
  await agent.nextFrame(); // complete
  await agent.send({ kind: 'transferResult', payload: { success: false, code: 'WRITE_FAILED', message: 'disk full' } });

  assert.deepEqual(await run, { success: false, code: 'WRITE_FAILED', message: 'disk full' });
});

test('a local read failure resolves with success: false without throwing', async () => {
  const files = [file(0, 'a.md', 1)];
  const { sender, agent } = setup(files, async () => {
    throw new Error('disk read error');
  });

  const run = sender.run();
  await agent.nextFrame(); // manifest
  await agent.send({ kind: 'transferAccepted', payload: { grantedBytes: 4 * 1024 * 1024 } });

  const outcome = await run;
  assert.equal(outcome.success, false);
  assert.match(outcome.message, /disk read error/);
});

test('sending waits for credit before a chunk that would exceed the window', async () => {
  const bigFile = new Uint8Array(300 * 1024).fill(1); // > FILE_CHUNK_BYTES (256 KiB), forces 2 chunks
  const files = [file(0, 'big.bin', bigFile.length)];
  const { sender, agent } = setup(files, async () => bigFile);

  const run = sender.run();
  await agent.nextFrame(); // manifest
  // Grant less than the whole file so the second chunk must wait for more credit.
  await agent.send({ kind: 'transferAccepted', payload: { grantedBytes: 256 * 1024 } });

  const firstChunk = await agent.nextFrame();
  assert.equal(firstChunk.kind, 'transferChunk');

  // The second chunk must not arrive yet - the window is exhausted.
  const raced = await Promise.race([
    agent.nextFrame().then(() => 'frame'),
    new Promise((resolve) => setTimeout(() => resolve('timeout'), 50)),
  ]);
  assert.equal(raced, 'timeout', 'sender must not exceed its granted credit');

  await agent.send({ kind: 'transferCredit', payload: { grantedBytes: bigFile.length } });

  const secondChunk = await agent.nextFrame();
  assert.equal(secondChunk.kind, 'transferChunk');
  await agent.nextFrame(); // fileEnd
  await agent.nextFrame(); // complete
  await agent.send({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });

  assert.deepEqual(await run, { success: true, code: null, message: '' });
});
