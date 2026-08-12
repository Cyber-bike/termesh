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
 *
 * `0x07`/`0x08`/`0x09` (`fsList`/`fsListResult`/`fsChanged`) are the
 * directory-tree panel's addition (candidate doc "目录树与双向文件传输",
 * phase 2A). They ride this same stream rather than a new ALPN - see
 * `agent/src/termstream.rs`'s module doc for why - so a bi-stream is
 * `fsList`-first instead of `open`-first. Failures reuse the existing
 * `error` frame (`FS_LIST_FAILED: ...`), matching how `SESSION_LIMIT_REACHED`
 * already works, rather than a dedicated error frame.
 *
 * `0x0A`..`0x10` (`transferManifest`..`transferResult`) are the note
 * transfer, also riding this connection for the same reason (see
 * `agent/src/termstream.rs`'s module doc): a chunk-and-credit protocol atop
 * `transfer.rs`'s already-built `TransferSession`, not doc §8.3's
 * `iroh-blobs` design - `iroh-blobs` isn't a dependency of the agent as of
 * this writing. `transferChunk` is, like `data`, not JSON: its payload is
 * `varint(fileIndex) + varint(offset) + raw bytes`.
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

export interface FsEntry {
  name: string;
  isDirectory: boolean;
}

export interface FsListPayload {
  path: string;
}

export interface FsListResultPayload {
  entries: FsEntry[];
}

/** "created" | "deleted" | "renamed" | "unknown" - see `FsChangedPayload` in `agent/src/termstream.rs`. */
export interface FsChangedPayload {
  kind: string;
}

export interface TransferEntry {
  index: number;
  relativePath: string;
  size: number;
}

export interface TransferManifestPayload {
  transferId: string;
  rootNote: string;
  entries: TransferEntry[];
  /** Doc §7.6: a session with a known cwd wins over the agent's configured receive root. */
  sessionId: string | null;
}

export interface TransferAcceptedPayload {
  grantedBytes: number;
}

/** Not JSON - see the module doc. */
export interface TransferChunkPayload {
  fileIndex: number;
  offset: number;
  data: Uint8Array;
}

export interface TransferFileEndPayload {
  fileIndex: number;
  sentSize: number;
}

export interface TransferCreditPayload {
  grantedBytes: number;
}

export type TransferCompletePayload = Record<string, never>;

export interface TransferResultPayload {
  success: boolean;
  code: string | null;
  message: string;
}

/** Handshake for the reverse direction (candidate doc phase 2B): the agent is the sender this time. */
export interface TransferPullRequestPayload {
  path: string;
}

/** Agent -> client, in reply to `transferPullRequest`. See `agent/src/termstream.rs`'s module doc for the rest of the flow. */
export interface TransferPullManifestPayload {
  entries: TransferEntry[];
}

export type TerminalStreamFrame =
  | { kind: 'open'; payload: OpenPayload }
  | { kind: 'opened'; payload: OpenedPayload }
  | { kind: 'error'; payload: ErrorPayload }
  /** Raw PTY bytes, either direction - the only frame that is not JSON. */
  | { kind: 'data'; payload: Uint8Array }
  | { kind: 'resize'; payload: ResizePayload }
  | { kind: 'shellEvent'; payload: ShellEventPayload }
  | { kind: 'close'; payload: ClosePayload }
  /** Handshake frame for a directory-tree stream, mutually exclusive with 'open' as the first frame. */
  | { kind: 'fsList'; payload: FsListPayload }
  | { kind: 'fsListResult'; payload: FsListResultPayload }
  | { kind: 'fsChanged'; payload: FsChangedPayload }
  /** Handshake frame for a transfer stream, mutually exclusive with 'open'/'fsList' as the first frame. */
  | { kind: 'transferManifest'; payload: TransferManifestPayload }
  | { kind: 'transferAccepted'; payload: TransferAcceptedPayload }
  | { kind: 'transferChunk'; payload: TransferChunkPayload }
  | { kind: 'transferFileEnd'; payload: TransferFileEndPayload }
  | { kind: 'transferCredit'; payload: TransferCreditPayload }
  | { kind: 'transferComplete'; payload: TransferCompletePayload }
  | { kind: 'transferResult'; payload: TransferResultPayload }
  /** Handshake frame for a pull stream, mutually exclusive with 'open'/'fsList'/'transferManifest' as the first frame. */
  | { kind: 'transferPullRequest'; payload: TransferPullRequestPayload }
  | { kind: 'transferPullManifest'; payload: TransferPullManifestPayload };

const KIND_OPEN = 0x00;
const KIND_DATA = 0x01;
const KIND_RESIZE = 0x02;
const KIND_SHELL_EVENT = 0x03;
const KIND_CLOSE = 0x04;
const KIND_OPENED = 0x05;
const KIND_ERROR = 0x06;
const KIND_FS_LIST = 0x07;
const KIND_FS_LIST_RESULT = 0x08;
const KIND_FS_CHANGED = 0x09;
const KIND_TRANSFER_MANIFEST = 0x0a;
const KIND_TRANSFER_ACCEPTED = 0x0b;
const KIND_TRANSFER_CHUNK = 0x0c;
const KIND_TRANSFER_FILE_END = 0x0d;
const KIND_TRANSFER_CREDIT = 0x0e;
const KIND_TRANSFER_COMPLETE = 0x0f;
const KIND_TRANSFER_RESULT = 0x10;
const KIND_TRANSFER_PULL_REQUEST = 0x11;
const KIND_TRANSFER_PULL_MANIFEST = 0x12;

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
    case 'fsList':
      return KIND_FS_LIST;
    case 'fsListResult':
      return KIND_FS_LIST_RESULT;
    case 'fsChanged':
      return KIND_FS_CHANGED;
    case 'transferManifest':
      return KIND_TRANSFER_MANIFEST;
    case 'transferAccepted':
      return KIND_TRANSFER_ACCEPTED;
    case 'transferChunk':
      return KIND_TRANSFER_CHUNK;
    case 'transferFileEnd':
      return KIND_TRANSFER_FILE_END;
    case 'transferCredit':
      return KIND_TRANSFER_CREDIT;
    case 'transferComplete':
      return KIND_TRANSFER_COMPLETE;
    case 'transferResult':
      return KIND_TRANSFER_RESULT;
    case 'transferPullRequest':
      return KIND_TRANSFER_PULL_REQUEST;
    case 'transferPullManifest':
      return KIND_TRANSFER_PULL_MANIFEST;
  }
}

function payloadBytes(frame: TerminalStreamFrame): Uint8Array {
  if (frame.kind === 'data') return frame.payload;
  if (frame.kind === 'transferChunk') return encodeTransferChunk(frame.payload);
  return textEncoder.encode(JSON.stringify(frame.payload));
}

/** `varint(fileIndex) + varint(offset) + raw bytes` (see the module doc). */
function encodeTransferChunk(payload: TransferChunkPayload): Uint8Array {
  const header: number[] = [];
  writeVarint(header, payload.fileIndex);
  writeVarint(header, payload.offset);
  const out = new Uint8Array(header.length + payload.data.length);
  out.set(header, 0);
  out.set(payload.data, header.length);
  return out;
}

function decodeTransferChunk(bytes: Uint8Array): TransferChunkPayload {
  const first = tryReadVarint(bytes, 0);
  if (!first) throw new TerminalStreamFrameError('transferChunk payload truncated (fileIndex)');
  const second = tryReadVarint(bytes, first.consumed);
  if (!second) throw new TerminalStreamFrameError('transferChunk payload truncated (offset)');
  return {
    fileIndex: first.value,
    offset: second.value,
    data: bytes.slice(first.consumed + second.consumed),
  };
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

function requireBoolean(raw: Record<string, unknown>, field: string): boolean {
  const value = raw[field];
  if (typeof value !== 'boolean') {
    throw new TerminalStreamFrameError(`missing or invalid "${field}"`);
  }
  return value;
}

function decodeFsEntries(raw: Record<string, unknown>): FsEntry[] {
  const value = raw.entries;
  if (!Array.isArray(value)) {
    throw new TerminalStreamFrameError('missing or invalid "entries"');
  }
  return value.map((item) => {
    if (typeof item !== 'object' || item === null) {
      throw new TerminalStreamFrameError('invalid entry in "entries"');
    }
    const entry = item as Record<string, unknown>;
    return { name: requireString(entry, 'name'), isDirectory: requireBoolean(entry, 'isDirectory') };
  });
}

function decodeTransferEntries(raw: Record<string, unknown>): TransferEntry[] {
  const value = raw.entries;
  if (!Array.isArray(value)) {
    throw new TerminalStreamFrameError('missing or invalid "entries"');
  }
  return value.map((item) => {
    if (typeof item !== 'object' || item === null) {
      throw new TerminalStreamFrameError('invalid entry in "entries"');
    }
    const entry = item as Record<string, unknown>;
    return {
      index: requireNumber(entry, 'index'),
      relativePath: requireString(entry, 'relativePath'),
      size: requireNumber(entry, 'size'),
    };
  });
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
    case KIND_FS_LIST: {
      const raw = decodeJson(payload);
      return { kind: 'fsList', payload: { path: requireString(raw, 'path') } };
    }
    case KIND_FS_LIST_RESULT: {
      const raw = decodeJson(payload);
      return { kind: 'fsListResult', payload: { entries: decodeFsEntries(raw) } };
    }
    case KIND_FS_CHANGED: {
      const raw = decodeJson(payload);
      return { kind: 'fsChanged', payload: { kind: requireString(raw, 'kind') } };
    }
    case KIND_TRANSFER_MANIFEST: {
      const raw = decodeJson(payload);
      return {
        kind: 'transferManifest',
        payload: {
          transferId: requireString(raw, 'transferId'),
          rootNote: requireString(raw, 'rootNote'),
          entries: decodeTransferEntries(raw),
          sessionId: optionalString(raw, 'sessionId'),
        },
      };
    }
    case KIND_TRANSFER_ACCEPTED: {
      const raw = decodeJson(payload);
      return { kind: 'transferAccepted', payload: { grantedBytes: requireNumber(raw, 'grantedBytes') } };
    }
    case KIND_TRANSFER_CHUNK:
      return { kind: 'transferChunk', payload: decodeTransferChunk(payload) };
    case KIND_TRANSFER_FILE_END: {
      const raw = decodeJson(payload);
      return {
        kind: 'transferFileEnd',
        payload: { fileIndex: requireNumber(raw, 'fileIndex'), sentSize: requireNumber(raw, 'sentSize') },
      };
    }
    case KIND_TRANSFER_CREDIT: {
      const raw = decodeJson(payload);
      return { kind: 'transferCredit', payload: { grantedBytes: requireNumber(raw, 'grantedBytes') } };
    }
    case KIND_TRANSFER_COMPLETE:
      decodeJson(payload); // validated for shape parity with the Rust side; the frame itself carries no fields
      return { kind: 'transferComplete', payload: {} };
    case KIND_TRANSFER_RESULT: {
      const raw = decodeJson(payload);
      return {
        kind: 'transferResult',
        payload: {
          success: requireBoolean(raw, 'success'),
          code: optionalString(raw, 'code'),
          message: requireString(raw, 'message'),
        },
      };
    }
    case KIND_TRANSFER_PULL_REQUEST: {
      const raw = decodeJson(payload);
      return { kind: 'transferPullRequest', payload: { path: requireString(raw, 'path') } };
    }
    case KIND_TRANSFER_PULL_MANIFEST: {
      const raw = decodeJson(payload);
      return { kind: 'transferPullManifest', payload: { entries: decodeTransferEntries(raw) } };
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
