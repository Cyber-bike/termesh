import * as assert from 'node:assert/strict';
import test from 'node:test';

import {
  TerminalStreamTransport,
  TerminalStreamError,
  type ByteStream,
} from './terminalStreamTransport.ts';
import {
  encodeTerminalStreamFrame,
  TerminalStreamFrameDecoder,
  type TerminalStreamFrame,
} from './terminalStreamFrame.ts';

/** Single-consumer async queue backing one direction of a stream pair. */
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

/** In-memory stand-in for one end of a `termy/terminal/1` bi-stream. */
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

/** The agent's end, driven frame-by-frame by each test. */
class FakeAgent {
  private readonly stream: ByteStream;
  private readonly decoder = new TerminalStreamFrameDecoder();

  constructor(stream: ByteStream) {
    this.stream = stream;
  }

  async send(frame: TerminalStreamFrame): Promise<void> {
    await this.stream.write(encodeTerminalStreamFrame(frame));
  }

  /** Sends a frame's bytes split into single-byte chunks, exercising the
   * transport's reassembly the way QUIC delivery actually behaves. */
  async sendFragmented(frame: TerminalStreamFrame): Promise<void> {
    for (const byte of encodeTerminalStreamFrame(frame)) {
      await this.stream.write(new Uint8Array([byte]));
    }
  }

  async nextFrame(): Promise<TerminalStreamFrame> {
    for (;;) {
      const frame = this.decoder.nextFrame();
      if (frame) return frame;
      const chunk = await this.stream.read();
      assert.ok(chunk !== null, 'the transport ended the stream unexpectedly');
      this.decoder.push(chunk);
    }
  }

  end(): void {
    this.stream.finishWrite();
  }
}

function setup() {
  const [controllerEnd, agentEnd] = streamPair();
  const transport = new TerminalStreamTransport(async () => controllerEnd);
  const agent = new FakeAgent(agentEnd);
  return { transport, agent };
}

async function openSession(transport: TerminalStreamTransport, agent: FakeAgent) {
  const openPromise = transport.open({ cols: 80, rows: 24 });
  const open = await agent.nextFrame();
  assert.deepEqual(open, { kind: 'open', payload: { cols: 80, rows: 24 } });
  await agent.send({ kind: 'opened', payload: { sessionId: 'session-1', shell: '/bin/bash' } });
  return openPromise;
}

test('open performs the doc 8.2 handshake and resolves with the session info', async () => {
  const { transport, agent } = setup();
  const info = await openSession(transport, agent);
  assert.deepEqual(info, { sessionId: 'session-1', shell: '/bin/bash' });
});

test('an error reply to open rejects with the parsed error code', async () => {
  const { transport, agent } = setup();
  const openPromise = transport.open({ cols: 80, rows: 24 });
  await agent.nextFrame();
  await agent.send({
    kind: 'error',
    payload: { message: 'SESSION_LIMIT_REACHED: at most 8 concurrent sessions' },
  });

  await assert.rejects(openPromise, (error: unknown) => {
    assert.ok(error instanceof TerminalStreamError);
    assert.equal(error.code, 'SESSION_LIMIT_REACHED');
    return true;
  });
});

test('the stream ending mid-handshake rejects open', async () => {
  const { transport, agent } = setup();
  const openPromise = transport.open({ cols: 80, rows: 24 });
  await agent.nextFrame();
  agent.end();

  await assert.rejects(openPromise, TerminalStreamError);
});

test('terminal output reaches onData, even fragmented byte by byte', async () => {
  const { transport, agent } = setup();
  const received: string[] = [];
  transport.onData((data) => received.push(new TextDecoder().decode(data)));
  await openSession(transport, agent);

  await agent.send({ kind: 'data', payload: new TextEncoder().encode('first ') });
  await agent.sendFragmented({ kind: 'data', payload: new TextEncoder().encode('second') });

  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.equal(received.join(''), 'first second');
});

test('write and resize become the right frames on the wire', async () => {
  const { transport, agent } = setup();
  await openSession(transport, agent);

  transport.write(new TextEncoder().encode('ls\n'));
  transport.resize(120, 40);

  assert.deepEqual(await agent.nextFrame(), {
    kind: 'data',
    payload: new TextEncoder().encode('ls\n'),
  });
  assert.deepEqual(await agent.nextFrame(), { kind: 'resize', payload: { cols: 120, rows: 40 } });
});

test('shell events are forwarded with the transport shape', async () => {
  const { transport, agent } = setup();
  const events: unknown[] = [];
  transport.onShellEvent((event) => events.push(event));
  await openSession(transport, agent);

  await agent.send({
    kind: 'shellEvent',
    payload: { event: 'command_end', source: 'osc633', cwd: '/tmp', exitCode: 3 },
  });
  await agent.send({
    kind: 'shellEvent',
    payload: { event: 'not-a-real-event', source: null, cwd: null, exitCode: null },
  });

  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(events, [{ type: 'command_end', source: 'osc633', exitCode: 3 }]);
});

test('a shell_exited close frame becomes an exit event with the code', async () => {
  const { transport, agent } = setup();
  const exits: unknown[] = [];
  transport.onExit((event) => exits.push(event));
  await openSession(transport, agent);

  await agent.send({ kind: 'close', payload: { reason: 'shell_exited', exitCode: 7 } });

  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(exits, [{ exitCode: 7, reason: 'shell_exited' }]);
});

test('the stream ending without a close frame reports peer_disconnected once', async () => {
  const { transport, agent } = setup();
  const exits: unknown[] = [];
  transport.onExit((event) => exits.push(event));
  await openSession(transport, agent);

  agent.end();

  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(exits, [{ exitCode: null, reason: 'peer_disconnected' }]);
});

test('a post-open error frame reaches onError and ends the session', async () => {
  const { transport, agent } = setup();
  const errors: unknown[] = [];
  const exits: unknown[] = [];
  transport.onError((code, message) => errors.push({ code, message }));
  transport.onExit((event) => exits.push(event));
  await openSession(transport, agent);

  await agent.send({ kind: 'error', payload: { message: 'PROTOCOL_ERROR: bad frame' } });

  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(errors, [{ code: 'PROTOCOL_ERROR', message: 'PROTOCOL_ERROR: bad frame' }]);
  assert.deepEqual(exits, [{ exitCode: null, reason: 'error' }]);
});

test('close sends a close frame and suppresses later exit events', async () => {
  const { transport, agent } = setup();
  const exits: unknown[] = [];
  transport.onExit((event) => exits.push(event));
  await openSession(transport, agent);

  await transport.close();

  assert.deepEqual(await agent.nextFrame(), {
    kind: 'close',
    payload: { reason: 'user', exitCode: null },
  });

  // Whatever the agent does afterwards must not resurface as an exit.
  agent.end();
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(exits, []);
});

test('opening twice is rejected', async () => {
  const { transport, agent } = setup();
  await openSession(transport, agent);
  await assert.rejects(transport.open({ cols: 80, rows: 24 }));
});

test('disposing a handler stops its deliveries', async () => {
  const { transport, agent } = setup();
  const received: string[] = [];
  const subscription = transport.onData((data) => received.push(new TextDecoder().decode(data)));
  await openSession(transport, agent);

  await agent.send({ kind: 'data', payload: new TextEncoder().encode('kept') });
  await new Promise((resolve) => setTimeout(resolve, 10));
  subscription.dispose();
  await agent.send({ kind: 'data', payload: new TextEncoder().encode('dropped') });
  await new Promise((resolve) => setTimeout(resolve, 10));

  assert.deepEqual(received, ['kept']);
});
