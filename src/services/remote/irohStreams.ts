/**
 * Adapters between `@number0/iroh` objects and the transport seams
 * (A0 verdict 2026-07-31: direct embedding - the binding loads and runs
 * inside Obsidian's Electron renderer).
 *
 * The binding is typed structurally here instead of importing its
 * declarations: the package is not a bundled dependency (esbuild marks it
 * external; the native module ships alongside the plugin - packaging is
 * 交接清单 step 5), so nothing in `src/` may import from it at build time.
 * These interfaces mirror the exact 1.1.0 API surface that the A0 scripts
 * exercised against the real library and the real agent.
 *
 * `ALPN_TERMINAL` must match `agent/src/p2p.rs`.
 */

import type { ByteStream } from './terminalStreamTransport.ts';

export const ALPN_TERMINAL = 'termy/terminal/1';

/** How many bytes one `recv.read()` may return at most. */
const READ_CHUNK_LIMIT = 64 * 1024;

/* Minimal structural mirror of the `@number0/iroh` 1.1.0 surface we use. */

export interface IrohSendStream {
  writeAll(bytes: number[]): Promise<void>;
  finish(): Promise<void>;
}

export interface IrohRecvStream {
  read(sizeLimit: number): Promise<number[] | null>;
}

export interface IrohBiStream {
  readonly send: IrohSendStream;
  readonly recv: IrohRecvStream;
}

export interface IrohConnection {
  openBi(): Promise<IrohBiStream>;
  close(errorCode: bigint, reason: number[]): void;
  closed(): Promise<string>;
}

export interface IrohEndpointAddr {
  id(): { toString(): string };
}

export interface IrohEndpointTicket {
  endpointAddr(): IrohEndpointAddr;
}

export interface IrohSecretKey {
  toBytes(): number[];
}

export interface IrohEndpointBuilder {
  secretKey(bytes: number[]): void;
  alpns(alpns: number[][]): void;
  relayMode(mode: unknown): void;
  bindAddr(addr: string): void;
  bind(): Promise<IrohEndpoint>;
}

export interface IrohEndpoint {
  connect(addr: IrohEndpointAddr, alpn: number[]): Promise<IrohConnection>;
  close(): Promise<void>;
}

/** The slice of `@number0/iroh`'s module surface the plugin consumes. */
export interface IrohModule {
  Endpoint: { builder(): IrohEndpointBuilder };
  EndpointTicket: { fromString(s: string): IrohEndpointTicket };
  RelayMode: { disabled(): unknown; defaultMode(): unknown };
  SecretKey: { generate(): IrohSecretKey; fromBytes(bytes: number[]): IrohSecretKey };
  presetN0(builder: IrohEndpointBuilder): void;
  presetMinimal(builder: IrohEndpointBuilder): void;
}

/**
 * Wraps one iroh bi-stream as the `ByteStream` the terminal transport
 * consumes. The binding speaks `Array<number>`; the seam speaks
 * `Uint8Array` - the copies here are the whole job.
 */
export function byteStreamFromBi(bi: IrohBiStream): ByteStream {
  return {
    async write(bytes: Uint8Array): Promise<void> {
      await bi.send.writeAll(Array.from(bytes));
    },
    async read(): Promise<Uint8Array | null> {
      const chunk = await bi.recv.read(READ_CHUNK_LIMIT);
      // The binding signals end-of-stream with an empty/absent chunk; a
      // healthy read never resolves with zero bytes.
      if (!chunk || chunk.length === 0) return null;
      return Uint8Array.from(chunk);
    },
    finishWrite(): void {
      void bi.send.finish().catch(() => {
        // The peer may already have closed the connection; finishing a dead
        // stream is a no-op, not an error worth surfacing.
      });
    },
  };
}

/**
 * The `openStream` factory `TerminalStreamTransport` expects: each call
 * opens a fresh bi-stream (= a fresh terminal session, doc 8.2) on the
 * device's existing connection.
 */
export function terminalStreamFactory(connection: IrohConnection): () => Promise<ByteStream> {
  return async () => byteStreamFromBi(await connection.openBi());
}
