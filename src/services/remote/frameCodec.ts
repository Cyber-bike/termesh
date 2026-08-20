/**
 * Binary frame codec for the remote protocol (doc 8.5).
 *
 * Third implementation of the same 38-byte header, after
 * `protocol/tools/frame-codec.js` and `protocol/generated/rust/src/frame.rs`.
 * All three are checked against the shared vectors in
 * `protocol/fixtures/frames/`, which is what keeps them from drifting.
 */

export const HEADER_BYTES = 38;
export const MAGIC_0 = 0x54; // 'T'
export const MAGIC_1 = 0x4d; // 'M'
export const FRAME_VERSION = 0x01;

export const KIND_TERMINAL_INPUT = 0x01;
export const KIND_TERMINAL_OUTPUT = 0x02;
export const KIND_FILE_CHUNK = 0x03;

export const TERMINAL_PAYLOAD_MAX = 32 * 1024;
export const FILE_PAYLOAD_MAX = 256 * 1024;
export const MESSAGE_MAX = HEADER_BYTES + FILE_PAYLOAD_MAX;
export const TERMINAL_FILE_INDEX = 0xffffffff;

export interface Frame {
  kind: number;
  /** sessionId for terminal frames, transferId for file chunks. */
  streamId: string;
  fileIndex: number;
  offset: number;
  payload: Uint8Array;
}

export class FrameError extends Error {
  readonly code = 'PROTOCOL_ERROR';

  constructor(message: string) {
    super(message);
    this.name = 'FrameError';
  }
}

export function payloadLimit(kind: number): number {
  return kind === KIND_FILE_CHUNK ? FILE_PAYLOAD_MAX : TERMINAL_PAYLOAD_MAX;
}

function uuidToBytes(uuid: string): Uint8Array {
  const hex = uuid.replace(/-/g, '').toLowerCase();
  if (!/^[0-9a-f]{32}$/.test(hex)) {
    throw new FrameError(`invalid streamId ${uuid}`);
  }
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i += 1) {
    bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function bytesToUuid(bytes: Uint8Array): string {
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export function encodeFrame(frame: Frame): Uint8Array {
  const { kind, payload } = frame;

  if (kind !== KIND_TERMINAL_INPUT && kind !== KIND_TERMINAL_OUTPUT && kind !== KIND_FILE_CHUNK) {
    throw new FrameError(`unknown kind 0x${kind.toString(16)}`);
  }
  if (payload.length > payloadLimit(kind)) {
    throw new FrameError(
      `payload ${payload.length} exceeds the limit for kind 0x${kind.toString(16)}`
    );
  }

  const out = new Uint8Array(HEADER_BYTES + payload.length);
  const view = new DataView(out.buffer);

  out[0] = MAGIC_0;
  out[1] = MAGIC_1;
  out[2] = FRAME_VERSION;
  out[3] = kind;
  out[4] = 0; // flags
  out[5] = 0; // reserved
  view.setUint32(6, payload.length, false);
  out.set(uuidToBytes(frame.streamId), 10);
  view.setUint32(26, kind === KIND_FILE_CHUNK ? frame.fileIndex : TERMINAL_FILE_INDEX, false);
  view.setBigUint64(30, BigInt(frame.offset), false);
  out.set(payload, HEADER_BYTES);

  return out;
}

export function decodeFrame(buf: Uint8Array): Frame {
  if (buf.length > MESSAGE_MAX) {
    throw new FrameError(`message ${buf.length} exceeds ${MESSAGE_MAX}`);
  }
  if (buf.length < HEADER_BYTES) {
    throw new FrameError(`truncated header: ${buf.length} < ${HEADER_BYTES}`);
  }
  if (buf[0] !== MAGIC_0 || buf[1] !== MAGIC_1) {
    throw new FrameError('bad magic');
  }
  if (buf[2] !== FRAME_VERSION) {
    throw new FrameError(`unsupported frame version ${buf[2]}`);
  }

  const kind = buf[3];
  if (kind !== KIND_TERMINAL_INPUT && kind !== KIND_TERMINAL_OUTPUT && kind !== KIND_FILE_CHUNK) {
    throw new FrameError(`unknown kind 0x${kind.toString(16)}`);
  }
  if (buf[4] !== 0) throw new FrameError('flags must be 0 in MVP');
  if (buf[5] !== 0) throw new FrameError('reserved byte must be 0');

  const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
  const payloadLength = view.getUint32(6, false);
  const actual = buf.length - HEADER_BYTES;
  if (payloadLength !== actual) {
    throw new FrameError(`payloadLength ${payloadLength} does not match the ${actual} bytes present`);
  }
  if (payloadLength > payloadLimit(kind)) {
    throw new FrameError(`payload ${payloadLength} exceeds the limit for kind 0x${kind.toString(16)}`);
  }

  const fileIndex = view.getUint32(26, false);
  if (kind === KIND_FILE_CHUNK) {
    if (fileIndex === TERMINAL_FILE_INDEX) {
      throw new FrameError('fileIndex 0xFFFFFFFF is reserved for terminal frames');
    }
  } else if (fileIndex !== TERMINAL_FILE_INDEX) {
    throw new FrameError('terminal frames must set fileIndex to 0xFFFFFFFF');
  }

  return {
    kind,
    streamId: bytesToUuid(buf.subarray(10, 26)),
    fileIndex,
    offset: Number(view.getBigUint64(30, false)),
    payload: buf.subarray(HEADER_BYTES),
  };
}
