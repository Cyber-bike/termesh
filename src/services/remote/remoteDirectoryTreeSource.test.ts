import * as assert from 'node:assert/strict';
import test from 'node:test';

import { RemoteDirectoryTreeError, RemoteDirectoryTreeSource } from './remoteDirectoryTreeSource.ts';
import type { ByteStream } from './terminalStreamTransport.ts';
import { encodeTerminalStreamFrame, TerminalStreamFrameDecoder, type TerminalStreamFrame } from './terminalStreamFrame.ts';

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

/** The agent's end of one fs stream, driven frame-by-frame by each test. */
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
      assert.ok(chunk !== null, 'the source ended the stream unexpectedly');
      this.decoder.push(chunk);
    }
  }

  /** Resolves once the client half-closes its write side (`finishWrite()`). */
  async waitForClientFinish(): Promise<void> {
    const chunk = await this.stream.read();
    assert.equal(chunk, null, 'expected the client to finish writing');
  }
}

/** Records one `FakeAgent` per stream the source opens, in open order. */
function setup(): { source: RemoteDirectoryTreeSource; agents: FakeAgent[] } {
  const agents: FakeAgent[] = [];
  const openStream = async (): Promise<ByteStream> => {
    const [clientEnd, agentEnd] = streamPair();
    agents.push(new FakeAgent(agentEnd));
    return clientEnd;
  };
  return { source: new RemoteDirectoryTreeSource(openStream), agents };
}

test('list() sends fsList and resolves with the entries from fsListResult', async () => {
  const { source, agents } = setup();
  const listPromise = source.list('/home/user/project');

  const request = await (async () => {
    // agents[0] only exists after openStream() runs, which happens
    // synchronously at the top of list() before the first await settles.
    await Promise.resolve();
    return agents[0].nextFrame();
  })();
  assert.deepEqual(request, { kind: 'fsList', payload: { path: '/home/user/project' } });

  await agents[0].send({
    kind: 'fsListResult',
    payload: {
      entries: [
        { name: 'src', isDirectory: true },
        { name: 'readme.md', isDirectory: false },
      ],
    },
  });

  const entries = await listPromise;
  assert.deepEqual(entries, [
    { name: 'src', isDirectory: true },
    { name: 'readme.md', isDirectory: false },
  ]);

  // list() must not leave the stream open once it has its answer.
  await agents[0].waitForClientFinish();
});

test('list() rejects with the agent-provided message on an error frame', async () => {
  const { source, agents } = setup();
  const listPromise = source.list('/no/such/path');
  await Promise.resolve();
  await agents[0].nextFrame();
  await agents[0].send({ kind: 'error', payload: { message: 'FS_LIST_FAILED: not found' } });

  await assert.rejects(listPromise, (error: unknown) => {
    assert.ok(error instanceof RemoteDirectoryTreeError);
    assert.equal(error.message, 'FS_LIST_FAILED: not found');
    return true;
  });
});

test('watch() delivers fsChanged notifications until disposed', async () => {
  const { source, agents } = setup();
  const changes: string[] = [];
  const disposable = source.watch('/home/user/project', (kind) => changes.push(kind));

  await Promise.resolve();
  await agents[0].nextFrame(); // the fsList handshake
  await agents[0].send({ kind: 'fsListResult', payload: { entries: [] } }); // ignored by watch()
  await agents[0].send({ kind: 'fsChanged', payload: { kind: 'unknown' } });

  await new Promise((resolve) => setTimeout(resolve, 20));
  assert.deepEqual(changes, ['unknown']);

  disposable.dispose();
  await agents[0].waitForClientFinish();

  await agents[0].send({ kind: 'fsChanged', payload: { kind: 'unknown' } }).catch(() => {});
  await new Promise((resolve) => setTimeout(resolve, 20));
  assert.deepEqual(changes, ['unknown'], 'no further changes after dispose');
});

test('an unrecognized change kind normalizes to "unknown"', async () => {
  const { source, agents } = setup();
  const changes: string[] = [];
  source.watch('/home/user/project', (kind) => changes.push(kind));

  await Promise.resolve();
  await agents[0].nextFrame();
  await agents[0].send({ kind: 'fsChanged', payload: { kind: 'something-new' } });

  await new Promise((resolve) => setTimeout(resolve, 20));
  assert.deepEqual(changes, ['unknown']);
});
