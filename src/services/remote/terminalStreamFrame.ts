/**
 * `termy/terminal/1` stream frame codec (implementation doc §8.2).
 *
 * TS mirror of `agent/src/termstream.rs` - the two must agree byte-for-byte
 * since they are the two ends of the same QUIC stream. Wire format: `kind (1
 * byte) + length (varint) + payload`. The doc fixes the kind byte for the
 * four post-handshake frames (`0x01 data`, `0x02 resize`, `0x03
 * shellEvent`, `0x04 close`) but leaves the handshake frames unassigned;
 * this module extends the same scheme to them (`0x00 open`, `0x05 opened`,
 * `0x06 error`), matching the choice made on the Rust side.
 *
 * Deliberately transport-agnostic, same as its Rust counterpart: it knows
 * nothing about `iroh` streams (that wiring is blocked on the A0 spike),
 * only how to turn frames into bytes and bytes back into frames.
 */

export interface OpenPayload {
  cols: number;
  rows: number;
}

export interface OpenedPayload {
  sessionId: string;
  shell: string;
}

export interface ErrorPayload {
  message: string;
}

export interface ResizePayload {
  cols: number;
  rows: number;
}

export interface ShellEventPayload {
  event: string;
  /** "osc133" | "osc633" - which integration emitted the event. */
  source: string | null;
  cwd: string | null;
  exitCode: number | null;
}

export interface ClosePayload {
  reason: string | null;
  /** Shell exit status when `reason` is "shell_exited" (doc 8.2). */
  exitCode: number | null;
}

export type TerminalStreamFrame =
  | { kind: 'open'; payload: OpenPayload }
  | { kind: 'opened'; payload: OpenedPayload }
  | { kind: 'error'; payload: ErrorPayload }
  /** Raw PTY bytes, either direction - the only frame that is not JSON. */
  | { kind: 'data'; payload: Uint8Array }
  | { kind: 'resize'; payload: ResizePayload }
  | { kind: 'shellEvent'; payload: ShellEventPayload }
  | { kind: 'close'; payload: ClosePayload };

const KIND_OPEN = 0x00;
const KIND_DATA = 0x01;
const KIND_RESIZE = 0x02;
const KIND_SHELL_EVENT = 0x03;
const KIND_CLOSE = 0x04;
const KIND_OPENED = 0x05;
const KIND_ERROR = 0x06;

/** Matches `MAX_FRAME_LEN` in `agent/src/termstream.rs`. */
const MAX_FRAME_LEN = 1024 * 1024;

export class TerminalStreamFrameError extends Error {
  readonly code = 'PROTOCOL_ERROR';

  constructor(message: string) {
    super(message);
    this.name = 'TerminalStreamFrameError';
  }
}

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

function kindByte(frame: TerminalStreamFrame): number {
  switch (frame.kind) {
    case 'open':
      return KIND_OPEN;
    case 'data':
      return KIND_DATA;
    case 'resize':
      return KIND_RESIZE;
    case 'shellEvent':
      return KIND_SHELL_EVENT;
    case 'close':
      return KIND_CLOSE;
    case 'opened':
      return KIND_OPENED;
    case 'error':
      return KIND_ERROR;
  }
}

function payloadBytes(frame: TerminalStreamFrame): Uint8Array {
  if (frame.kind === 'data') return frame.payload;
  return textEncoder.encode(JSON.stringify(frame.payload));
}

function writeVarint(out: number[], value: number): void {
  let v = value >>> 0;
  for (;;) {
    const byte = v & 0x7f;
    v >>>= 7;
    if (v === 0) {
      out.push(byte);
      return;
    }
    out.push(byte | 0x80);
  }
}

/**
 * Returns `{ value, consumed }` if `bytes` starting at `offset` holds a
 * complete varint, `null` if it might still be incomplete (more bytes
 * needed). Ten continuation bytes is already far more than any value under
 * `MAX_FRAME_LEN` needs, so that case is a protocol violation rather than
 * "wait for more data" - same bound as the Rust decoder.
 */
function tryReadVarint(bytes: Uint8Array, offset: number): { value: number; consumed: number } | null {
  let value = 0;
  let shift = 0;
  for (let i = offset; i < bytes.length; i += 1) {
    if (i - offset >= 10) {
      throw new TerminalStreamFrameError('termy/terminal/1 frame length prefix is too long');
    }
    const byte = bytes[i];
    value |= (byte & 0x7f) << shift;
    if ((byte & 0x80) === 0) {
      return { value: value >>> 0, consumed: i - offset + 1 };
    }
    shift += 7;
  }
  return null;
}

/** Serialises a frame to the bytes that should be written to the stream. */
export function encodeTerminalStreamFrame(frame: TerminalStreamFrame): Uint8Array {
  const payload = payloadBytes(frame);
  const lenBytes: number[] = [];
  writeVarint(lenBytes, payload.length);

  const out = new Uint8Array(1 + lenBytes.length + payload.length);
  out[0] = kindByte(frame);
  out.set(lenBytes, 1);
  out.set(payload, 1 + lenBytes.length);
  return out;
}

function decodeJson(payload: Uint8Array): Record<string, unknown> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(textDecoder.decode(payload));
  } catch (error) {
    throw new TerminalStreamFrameError(`cannot decode frame payload: ${(error as Error).message}`);
  }
  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    throw new TerminalStreamFrameError('frame payload must be a JSON object');
  }
  return parsed as Record<string, unknown>;
}

function requireNumber(raw: Record<string, unknown>, field: string): number {
  const value = raw[field];
  if (typeof value !== 'number') {
    throw new TerminalStreamFrameError(`missing or invalid "${field}"`);
  }
  return value;
}

function requireString(raw: Record<string, unknown>, field: string): string {
  const value = raw[field];
  if (typeof value !== 'string') {
    throw new TerminalStreamFrameError(`missing or invalid "${field}"`);
  }
  return value;
}

function optionalString(raw: Record<string, unknown>, field: string): string | null {
  const value = raw[field];
  return typeof value === 'string' ? value : null;
}

function optionalNumber(raw: Record<string, unknown>, field: string): number | null {
  const value = raw[field];
  return typeof value === 'number' ? value : null;
}

function decodeFrameParts(kind: number, payload: Uint8Array): TerminalStreamFrame {
  switch (kind) {
    case KIND_OPEN: {
      const raw = decodeJson(payload);
      return { kind: 'open', payload: { cols: requireNumber(raw, 'cols'), rows: requireNumber(raw, 'rows') } };
    }
    case KIND_DATA:
      return { kind: 'data', payload };
    case KIND_RESIZE: {
      const raw = decodeJson(payload);
      return { kind: 'resize', payload: { cols: requireNumber(raw, 'cols'), rows: requireNumber(raw, 'rows') } };
    }
    case KIND_SHELL_EVENT: {
      const raw = decodeJson(payload);
      return {
        kind: 'shellEvent',
        payload: {
          event: requireString(raw, 'event'),
          source: optionalString(raw, 'source'),
          cwd: optionalString(raw, 'cwd'),
          exitCode: optionalNumber(raw, 'exitCode'),
        },
      };
    }
    case KIND_CLOSE: {
      const raw = decodeJson(payload);
      return {
        kind: 'close',
        payload: {
          reason: optionalString(raw, 'reason'),
          exitCode: optionalNumber(raw, 'exitCode'),
        },
      };
    }
    case KIND_OPENED: {
      const raw = decodeJson(payload);
      return {
        kind: 'opened',
        payload: { sessionId: requireString(raw, 'sessionId'), shell: requireString(raw, 'shell') },
      };
    }
    case KIND_ERROR: {
      const raw = decodeJson(payload);
      return { kind: 'error', payload: { message: requireString(raw, 'message') } };
    }
    default:
      throw new TerminalStreamFrameError(
        `unknown termy/terminal/1 frame kind 0x${kind.toString(16).padStart(2, '0')}`
      );
  }
}

/**
 * Accumulates bytes arriving from a `termy/terminal/1` stream and pops
 * complete frames off the front. Feed it whatever chunks the underlying
 * reader hands back, in whatever sizes they happen to arrive in - it makes
 * no assumption that a `push()` call lines up with a frame boundary.
 */
export class TerminalStreamFrameDecoder {
  private chunks: Uint8Array[] = [];
  private length = 0;

  push(bytes: Uint8Array): void {
    if (bytes.length === 0) return;
    this.chunks.push(bytes);
    this.length += bytes.length;
  }

  private buffer(): Uint8Array {
    if (this.chunks.length <= 1) {
      return this.chunks[0] ?? new Uint8Array(0);
    }
    const merged = new Uint8Array(this.length);
    let offset = 0;
    for (const chunk of this.chunks) {
      merged.set(chunk, offset);
      offset += chunk.length;
    }
    this.chunks = [merged];
    return merged;
  }

  /**
   * Pops one frame if the buffer already holds a complete one. `null` means
   * "not enough bytes yet" and is not an error - read more from the stream
   * and call this again.
   */
  nextFrame(): TerminalStreamFrame | null {
    if (this.length === 0) return null;
    const buf = this.buffer();

    const kind = buf[0];
    const varint = tryReadVarint(buf, 1);
    if (!varint) return null;
    const { value: len, consumed } = varint;
    if (len > MAX_FRAME_LEN) {
      throw new TerminalStreamFrameError(
        `termy/terminal/1 frame of ${len} bytes exceeds the ${MAX_FRAME_LEN}-byte limit`
      );
    }

    const headerLen = 1 + consumed;
    const totalLen = headerLen + len;
    if (buf.length < totalLen) return null;

    const payload = buf.slice(headerLen, totalLen);
    const rest = buf.subarray(totalLen);
    this.chunks = rest.length > 0 ? [rest] : [];
    this.length = rest.length;

    return decodeFrameParts(kind, payload);
  }
}
