'use strict';

/**
 * Reference codec for the 38-byte binary frame header (doc 8.5).
 *
 * This is the normative implementation the Rust side is checked against: both
 * ends run fixtures/frames/*.hex through their own decoder and must agree on
 * accept/reject and on the decoded field values.
 */

const MAGIC_0 = 0x54; // 'T'
const MAGIC_1 = 0x4d; // 'M'
const VERSION = 0x01;
const HEADER_BYTES = 38;

const KIND_TERMINAL_INPUT = 0x01;
const KIND_TERMINAL_OUTPUT = 0x02;
const KIND_FILE_CHUNK = 0x03;

const TERMINAL_PAYLOAD_MAX = 32768; // 32 KiB
const FILE_PAYLOAD_MAX = 262144; // 256 KiB
const MESSAGE_MAX = HEADER_BYTES + FILE_PAYLOAD_MAX; // 262182
const TERMINAL_FILE_INDEX = 0xffffffff;

class FrameError extends Error {
  constructor(reason) {
    super(reason);
    this.name = 'FrameError';
    this.code = 'PROTOCOL_ERROR';
  }
}

function payloadLimitFor(kind) {
  return kind === KIND_FILE_CHUNK ? FILE_PAYLOAD_MAX : TERMINAL_PAYLOAD_MAX;
}

/**
 * @param {{kind:number, streamId:string, fileIndex:number, offset:bigint|number, payload:Buffer}} frame
 * @returns {Buffer}
 */
function encode(frame) {
  const { kind, streamId, payload } = frame;
  const fileIndex = kind === KIND_FILE_CHUNK ? frame.fileIndex : TERMINAL_FILE_INDEX;
  const offset = BigInt(frame.offset);

  if (kind !== KIND_TERMINAL_INPUT && kind !== KIND_TERMINAL_OUTPUT && kind !== KIND_FILE_CHUNK) {
    throw new FrameError(`unknown kind 0x${kind.toString(16)}`);
  }
  if (payload.length > payloadLimitFor(kind)) {
    throw new FrameError(`payload ${payload.length} exceeds the limit for kind 0x${kind.toString(16)}`);
  }

  const header = Buffer.alloc(HEADER_BYTES);
  header[0] = MAGIC_0;
  header[1] = MAGIC_1;
  header[2] = VERSION;
  header[3] = kind;
  header[4] = 0; // flags, fixed 0 in MVP
  header[5] = 0; // reserved, fixed 0
  header.writeUInt32BE(payload.length, 6);
  uuidToBytes(streamId).copy(header, 10);
  header.writeUInt32BE(fileIndex, 26);
  header.writeBigUInt64BE(offset, 30);

  return Buffer.concat([header, payload]);
}

/**
 * @param {Buffer} buf
 * @returns {{kind:number, streamId:string, fileIndex:number, offset:bigint, payload:Buffer}}
 * @throws {FrameError}
 */
function decode(buf) {
  if (buf.length > MESSAGE_MAX) throw new FrameError(`message ${buf.length} exceeds ${MESSAGE_MAX}`);
  if (buf.length < HEADER_BYTES) throw new FrameError(`truncated header: ${buf.length} < ${HEADER_BYTES}`);
  if (buf[0] !== MAGIC_0 || buf[1] !== MAGIC_1) throw new FrameError('bad magic');
  if (buf[2] !== VERSION) throw new FrameError(`unsupported frame version ${buf[2]}`);

  const kind = buf[3];
  if (kind !== KIND_TERMINAL_INPUT && kind !== KIND_TERMINAL_OUTPUT && kind !== KIND_FILE_CHUNK) {
    throw new FrameError(`unknown kind 0x${kind.toString(16)}`);
  }
  if (buf[4] !== 0) throw new FrameError('flags must be 0 in MVP');
  if (buf[5] !== 0) throw new FrameError('reserved byte must be 0');

  const payloadLength = buf.readUInt32BE(6);
  const actual = buf.length - HEADER_BYTES;
  if (payloadLength !== actual) {
    throw new FrameError(`payloadLength ${payloadLength} does not match the ${actual} bytes present`);
  }
  if (payloadLength > payloadLimitFor(kind)) {
    throw new FrameError(`payload ${payloadLength} exceeds the limit for kind 0x${kind.toString(16)}`);
  }

  const fileIndex = buf.readUInt32BE(26);
  if (kind === KIND_FILE_CHUNK) {
    if (fileIndex > 255) throw new FrameError(`fileIndex ${fileIndex} exceeds the 256-file batch limit`);
  } else if (fileIndex !== TERMINAL_FILE_INDEX) {
    throw new FrameError('terminal frames must set fileIndex to 0xFFFFFFFF');
  }

  return {
    kind,
    streamId: bytesToUuid(buf.subarray(10, 26)),
    fileIndex,
    offset: buf.readBigUInt64BE(30),
    payload: buf.subarray(HEADER_BYTES)
  };
}

/**
 * Per-counting-domain offset continuity (doc 8.5).
 *
 * File frames are keyed by (transferId, fileIndex) and a gap is fatal; terminal
 * frames are keyed by (sessionId, kind) and a gap is only reported, because
 * killing a live terminal over an ordering check that WSS already guarantees is
 * not worth it.
 */
class OffsetTracker {
  constructor() {
    this.domains = new Map();
  }

  domainKey(frame) {
    return frame.kind === KIND_FILE_CHUNK
      ? `${frame.streamId}/${frame.fileIndex}`
      : `${frame.streamId}/kind${frame.kind}`;
  }

  /** @returns {{ok:boolean, fatal:boolean, expected:bigint, got:bigint}} */
  accept(frame) {
    const key = this.domainKey(frame);
    const expected = this.domains.get(key) || 0n;
    const got = BigInt(frame.offset);
    const ok = expected === got;
    if (ok) this.domains.set(key, expected + BigInt(frame.payload.length));
    return { ok, fatal: !ok && frame.kind === KIND_FILE_CHUNK, expected, got };
  }
}

function uuidToBytes(uuid) {
  const hex = String(uuid).replace(/-/g, '');
  if (!/^[0-9a-f]{32}$/.test(hex)) throw new FrameError(`invalid streamId ${uuid}`);
  return Buffer.from(hex, 'hex');
}

function bytesToUuid(bytes) {
  const h = Buffer.from(bytes).toString('hex');
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

module.exports = {
  encode,
  decode,
  OffsetTracker,
  FrameError,
  HEADER_BYTES,
  MESSAGE_MAX,
  TERMINAL_PAYLOAD_MAX,
  FILE_PAYLOAD_MAX,
  TERMINAL_FILE_INDEX,
  KIND_TERMINAL_INPUT,
  KIND_TERMINAL_OUTPUT,
  KIND_FILE_CHUNK
};
