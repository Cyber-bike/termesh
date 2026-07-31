/**
 * v2.0 remote terminal transport (doc 6.3): `TerminalTransport` implemented
 * over one `termy/terminal/1` bi-stream, speaking the doc 8.2 frame
 * protocol via `terminalStreamFrame.ts`.
 *
 * The stream itself is behind the tiny [`ByteStream`] seam: the A0 spike
 * decides how a real iroh `BiStream` is obtained (direct embedding vs
 * termy-bridge), but everything above the raw bytes - handshake, frame
 * routing, event fan-out, teardown - is identical either way, so it is
 * built and tested now against in-memory streams. The agent-side behaviour
 * this pairs with is `agent/src/serve.rs`, which passes the same protocol
 * over genuine QUIC in its integration tests.
 *
 * One instance serves one session, matching the interface contract in
 * `transport.ts`.
 */

import type {
  Disposable,
  ShellEventPayload,
  TerminalExitEvent,
  TerminalOpenOptions,
  TerminalSessionInfo,
  TerminalTransport,
} from './transport';
import { toDisposable } from './transport';
import {
  encodeTerminalStreamFrame,
  TerminalStreamFrameDecoder,
  type ClosePayload,
  type ShellEventPayload as WireShellEvent,
  type TerminalStreamFrame,
} from './terminalStreamFrame';

/**
 * The one thing the iroh integration must eventually provide: an open
 * bidirectional byte stream. Maps 1:1 onto `@number0/iroh`'s `BiStream`
 * (`send.writeAll`/`send.finish` and `recv.read`).
 */
export interface ByteStream {
  write(bytes: Uint8Array): Promise<void>;
  /** Resolves with the next chunk, or `null` once the peer finished. */
  read(): Promise<Uint8Array | null>;
  /** No more writes (QUIC `finish`). Reading may continue. */
  finishWrite(): void;
}

/** `SESSION_LIMIT_REACHED: at most 8...` -> `SESSION_LIMIT_REACHED`. */
function errorCode(message: string): string {
  const match = /^([A-Z][A-Z0-9_]*):/.exec(message);
  return match ? match[1] : 'PROTOCOL_ERROR';
}

export class TerminalStreamError extends Error {
  readonly code: string;

  constructor(message: string) {
    super(message);
    this.name = 'TerminalStreamError';
    this.code = errorCode(message);
  }
}

const SHELL_EVENT_TYPES = new Set(['prompt_start', 'command_start', 'command_executed', 'command_end']);
const SHELL_EVENT_SOURCES = new Set(['osc133', 'osc633']);

export class TerminalStreamTransport implements TerminalTransport {
  private readonly openStream: () => Promise<ByteStream>;
  private stream: ByteStream | null = null;
  private opened = false;
  private closed = false;
  private readonly dataHandlers = new Set<(data: Uint8Array) => void>();
  private readonly exitHandlers = new Set<(event: TerminalExitEvent) => void>();
  private readonly errorHandlers = new Set<(code: string, message: string) => void>();
  private readonly shellEventHandlers = new Set<(event: ShellEventPayload) => void>();

  constructor(openStream: () => Promise<ByteStream>) {
    this.openStream = openStream;
  }

  async open(options: TerminalOpenOptions): Promise<TerminalSessionInfo> {
    if (this.opened) throw new Error('this transport already opened its session');
    this.opened = true;

    const stream = await this.openStream();
    this.stream = stream;

    await stream.write(
      encodeTerminalStreamFrame({
        kind: 'open',
        payload: { cols: options.cols, rows: options.rows },
      }),
    );

    // The first frame decides the session's fate: `opened` starts it,
    // `error` is doc 8.2's refusal (limit reached, shell failed, ...).
    const decoder = new TerminalStreamFrameDecoder();
    for (;;) {
      const frame = decoder.nextFrame();
      if (frame) {
        if (frame.kind === 'opened') {
          void this.pump(stream, decoder);
          return { sessionId: frame.payload.sessionId, shell: frame.payload.shell };
        }
        if (frame.kind === 'error') {
          this.closed = true;
          throw new TerminalStreamError(frame.payload.message);
        }
        this.closed = true;
        throw new TerminalStreamError(`PROTOCOL_ERROR: expected opened, got ${frame.kind}`);
      }
      const chunk = await stream.read();
      if (chunk === null) {
        this.closed = true;
        throw new TerminalStreamError('PROTOCOL_ERROR: the agent closed the stream during the handshake');
      }
      decoder.push(chunk);
    }
  }

  write(data: Uint8Array): void {
    this.send({ kind: 'data', payload: data });
  }

  resize(cols: number, rows: number): void {
    this.send({ kind: 'resize', payload: { cols, rows } });
  }

  async close(reason?: string): Promise<void> {
    if (this.closed || !this.stream) {
      this.closed = true;
      return;
    }
    this.closed = true;
    try {
      await this.stream.write(
        encodeTerminalStreamFrame({
          kind: 'close',
          payload: { reason: reason ?? 'user', exitCode: null },
        }),
      );
      this.stream.finishWrite();
    } catch {
      // The stream is already gone; closing it was the goal anyway.
    }
  }

  onData(handler: (data: Uint8Array) => void): Disposable {
    this.dataHandlers.add(handler);
    return toDisposable(() => this.dataHandlers.delete(handler));
  }

  onExit(handler: (event: TerminalExitEvent) => void): Disposable {
    this.exitHandlers.add(handler);
    return toDisposable(() => this.exitHandlers.delete(handler));
  }

  onError(handler: (code: string, message: string) => void): Disposable {
    this.errorHandlers.add(handler);
    return toDisposable(() => this.errorHandlers.delete(handler));
  }

  onShellEvent(handler: (event: ShellEventPayload) => void): Disposable {
    this.shellEventHandlers.add(handler);
    return toDisposable(() => this.shellEventHandlers.delete(handler));
  }

  private send(frame: TerminalStreamFrame): void {
    if (this.closed || !this.stream) return;
    this.stream.write(encodeTerminalStreamFrame(frame)).catch((error: unknown) => {
      this.emitError('PROTOCOL_ERROR', `cannot write to the session stream: ${String(error)}`);
    });
  }

  /** Reads frames until the stream ends or a `close` frame arrives. */
  private async pump(stream: ByteStream, decoder: TerminalStreamFrameDecoder): Promise<void> {
    try {
      for (;;) {
        let frame = decoder.nextFrame();
        while (frame) {
          if (!this.route(frame)) return;
          frame = decoder.nextFrame();
        }
        const chunk = await stream.read();
        if (chunk === null) {
          // The stream ended without a close frame: the connection (or the
          // agent) went away underneath the session.
          this.finish({ exitCode: null, reason: 'peer_disconnected' });
          return;
        }
        decoder.push(chunk);
      }
    } catch (error) {
      if (this.closed) return;
      this.emitError('PROTOCOL_ERROR', String(error));
      this.finish({ exitCode: null, reason: 'error' });
    }
  }

  /** Returns false once the session is over and pumping must stop. */
  private route(frame: TerminalStreamFrame): boolean {
    switch (frame.kind) {
      case 'data':
        for (const handler of this.dataHandlers) handler(frame.payload);
        return true;
      case 'shellEvent':
        this.routeShellEvent(frame.payload);
        return true;
      case 'close':
        this.finish(exitEventFromClose(frame.payload));
        return false;
      case 'error':
        this.emitError(errorCode(frame.payload.message), frame.payload.message);
        this.finish({ exitCode: null, reason: 'error' });
        return false;
      default:
        // open/opened/resize only ever travel the other way; tolerate them
        // rather than killing a healthy session over a peer bug.
        return true;
    }
  }

  private routeShellEvent(payload: WireShellEvent): void {
    if (!SHELL_EVENT_TYPES.has(payload.event)) return;
    const source = payload.source !== null && SHELL_EVENT_SOURCES.has(payload.source)
      ? payload.source
      : 'osc133';
    const event: ShellEventPayload = {
      type: payload.event as ShellEventPayload['type'],
      source: source as ShellEventPayload['source'],
      exitCode: payload.exitCode,
    };
    for (const handler of this.shellEventHandlers) handler(event);
  }

  private finish(event: TerminalExitEvent): void {
    if (this.closed) return;
    this.closed = true;
    for (const handler of this.exitHandlers) handler(event);
  }

  private emitError(code: string, message: string): void {
    for (const handler of this.errorHandlers) handler(code, message);
  }
}

function exitEventFromClose(payload: ClosePayload): TerminalExitEvent {
  const reason: TerminalExitEvent['reason'] =
    payload.reason === 'shell_exited' ? 'shell_exited'
    : payload.reason === 'user' ? 'user'
    : payload.reason === 'error' ? 'error'
    : 'peer_disconnected';
  return { exitCode: payload.exitCode, reason };
}
