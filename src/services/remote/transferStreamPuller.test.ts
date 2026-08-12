import * as assert from 'node:assert/strict';
import test from 'node:test';

import { TransferStreamPuller } from './transferStreamPuller.ts';
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

/** The agent's end of one pull stream, driven frame-by-frame by each test. */
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
      assert.ok(chunk !== null, 'the puller ended the stream unexpectedly');
      this.decoder.push(chunk);
    }
  }
}

function setup(path: string) {
  const [clientEnd, agentEnd] = streamPair();
  const agent = new FakeAgent(agentEnd);
  const puller = new TransferStreamPuller(async () => clientEnd, path);
  return { puller, agent };
}

test('pulls a single file and resolves with its content', async () => {
  const { puller, agent } = setup('/home/user/project/demo.md');
  const run = puller.run();

  assert.deepEqual(await agent.nextFrame(), {
    kind: 'transferPullRequest',
    payload: { path: '/home/user/project/demo.md' },
  });
  await agent.send({
    kind: 'transferPullManifest',
    payload: { entries: [{ index: 0, relativePath: 'demo.md', size: 11 }] },
  });

  const credit = await agent.nextFrame();
  assert.equal(credit.kind, 'transferCredit');

  await agent.send({
    kind: 'transferChunk',
    payload: { fileIndex: 0, offset: 0, data: new TextEncoder().encode('hello world') },
  });
  await agent.send({ kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 11 } });
  await agent.send({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });

  const outcome = await run;
  assert.equal(outcome.success, true);
  assert.equal(outcome.files.length, 1);
  assert.equal(outcome.files[0].relativePath, 'demo.md');
  assert.equal(new TextDecoder().decode(outcome.files[0].data), 'hello world');
});

test('pulls a directory with multiple files, reassembling each from its own chunks', async () => {
  const { puller, agent } = setup('/home/user/project');
  const run = puller.run();

  await agent.nextFrame(); // request
  await agent.send({
    kind: 'transferPullManifest',
    payload: {
      entries: [
        { index: 0, relativePath: 'project/readme.md', size: 5 },
        { index: 1, relativePath: 'project/assets/img.png', size: 4 },
      ],
    },
  });
  await agent.nextFrame(); // initial credit

  await agent.send({
    kind: 'transferChunk',
    payload: { fileIndex: 0, offset: 0, data: new TextEncoder().encode('ab') },
  });
  await agent.send({
    kind: 'transferChunk',
    payload: { fileIndex: 0, offset: 2, data: new TextEncoder().encode('cde') },
  });
  await agent.send({ kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 5 } });

  await agent.send({
    kind: 'transferChunk',
    payload: { fileIndex: 1, offset: 0, data: Uint8Array.from([0x89, 0x50, 0x4e, 0x47]) },
  });
  await agent.send({ kind: 'transferFileEnd', payload: { fileIndex: 1, sentSize: 4 } });
  await agent.send({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });

  const outcome = await run;
  assert.equal(outcome.success, true);
  assert.equal(outcome.files.length, 2);
  assert.equal(new TextDecoder().decode(outcome.files[0].data), 'abcde');
  assert.deepEqual(Array.from(outcome.files[1].data), [0x89, 0x50, 0x4e, 0x47]);
});

test('a rejected pull request resolves with the agent-provided error', async () => {
  const { puller, agent } = setup('/no/such/path');
  const run = puller.run();
  await agent.nextFrame();
  await agent.send({ kind: 'error', payload: { message: 'PULL_FAILED: no such file' } });

  const outcome = await run;
  assert.deepEqual(outcome, { success: false, code: 'PULL_FAILED', message: 'PULL_FAILED: no such file', files: [] });
});

test('a byte-count mismatch at fileEnd is a protocol error, not a silently wrong file', async () => {
  const { puller, agent } = setup('/home/user/project/demo.md');
  const run = puller.run();
  await agent.nextFrame();
  await agent.send({
    kind: 'transferPullManifest',
    payload: { entries: [{ index: 0, relativePath: 'demo.md', size: 11 }] },
  });
  await agent.nextFrame();

  await agent.send({
    kind: 'transferChunk',
    payload: { fileIndex: 0, offset: 0, data: new TextEncoder().encode('short') },
  });
  await agent.send({ kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: 11 } });

  const outcome = await run;
  assert.equal(outcome.success, false);
  assert.equal(outcome.code, 'PROTOCOL_ERROR');
});

test('credit is topped up as bytes arrive for a file larger than the initial grant', async () => {
  const { puller, agent } = setup('/home/user/project/big.bin');
  const run = puller.run();
  await agent.nextFrame(); // request
  await agent.send({
    kind: 'transferPullManifest',
    payload: { entries: [{ index: 0, relativePath: 'big.bin', size: 5 * 1024 * 1024 }] },
  });

  const firstCredit = await agent.nextFrame();
  assert.equal(firstCredit.kind, 'transferCredit');
  const firstGrant = firstCredit.kind === 'transferCredit' ? firstCredit.payload.grantedBytes : 0;
  assert.ok(firstGrant > 0);

  // Send just under the initial grant, split into realistically-sized
  // chunks (a single frame is capped at MAX_FRAME_LEN = 1 MiB - real
  // senders, Rust and TS alike, chunk well below that).
  const CHUNK_SIZE = 256 * 1024;
  const total = 5 * 1024 * 1024;
  let offset = 0;
  let toppedUp = false;
  while (offset < total) {
    const size = Math.min(CHUNK_SIZE, total - offset);
    await agent.send({
      kind: 'transferChunk',
      payload: { fileIndex: 0, offset, data: new Uint8Array(size).fill(9) },
    });
    offset += size;

    if (!toppedUp && offset + 1024 * 1024 >= firstGrant) {
      const topUp = await agent.nextFrame();
      assert.equal(topUp.kind, 'transferCredit');
      const topUpGrant = topUp.kind === 'transferCredit' ? topUp.payload.grantedBytes : 0;
      assert.ok(topUpGrant > firstGrant, 'the grant must increase once received bytes approach it');
      toppedUp = true;
    }
  }
  assert.ok(toppedUp, 'the test must actually have crossed the top-up threshold');

  await agent.send({ kind: 'transferFileEnd', payload: { fileIndex: 0, sentSize: total } });
  await agent.send({ kind: 'transferResult', payload: { success: true, code: null, message: '' } });

  const outcome = await run;
  assert.equal(outcome.success, true);
  assert.equal(outcome.files[0].data.length, total);
});
