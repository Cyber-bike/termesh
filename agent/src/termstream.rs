//! `termy/terminal/1` stream frame codec (implementation doc §8.2).
//!
//! Each `termy/terminal/1` stream carries, in order: one `Open` frame from
//! the control end, then either `Opened` or `Error` from the agent, then an
//! unbounded sequence of `Data`/`Resize`/`ShellEvent`/`Close` frames until
//! the stream closes. QUIC already guarantees delivery order and stream
//! identity, so - unlike the V1 binary protocol in
//! `protocol/generated/rust/src/frame.rs` - frames need no `magic`/
//! `version`/`streamId`/`offset` fields.
//!
//! Wire format: `kind (1 byte) + length (varint) + payload`. The doc fixes
//! the kind byte for the four post-handshake frames (`0x01 data`, `0x02
//! resize`, `0x03 shellEvent`, `0x04 close`) but leaves the handshake
//! frames unassigned; this module extends the same scheme to them (`0x00
//! open`, `0x05 opened`, `0x06 error`) so a stream is one uniform frame
//! sequence rather than two different framings back to back.
//!
//! This module is deliberately transport-agnostic: it does not read from an
//! `iroh` stream directly (that wiring is blocked on the A0 spike), only
//! from byte slices. `FrameDecoder` accumulates bytes handed to it and pops
//! complete frames, which is the same shape needed later whether those
//! bytes arrive synchronously (as in the tests below) or from polling an
//! async `iroh::endpoint::RecvStream`.
//!
//! `0x07`/`0x08`/`0x09` (`FsList`/`FsListResult`/`FsChanged`) are the
//! directory-tree panel's addition (candidate doc "目录树与双向文件传输",
//! phase 2A). These do NOT get their own ALPN/connection: doc §8.1 already
//! notes "one connection negotiates exactly one ALPN, so a transfer needs
//! its own connection" - `ControllerGate` (`p2p.rs`) admits exactly one
//! *connection* at a time, not one *peer*, so a second ALPN from the same
//! already-admitted controller would either bypass the gate entirely (a
//! real hole) or be refused as a second controller (making the feature
//! unusable while a terminal is open). Riding the existing
//! `termy/terminal/1` connection as another kind of bi-stream sidesteps
//! that problem entirely: a stream is `FsList`-first instead of `Open`-first,
//! and `serve.rs` dispatches on whichever handshake frame arrives. Failures
//! reuse the existing `Error` frame (`FS_LIST_FAILED: ...`) rather than a
//! dedicated error frame, matching how `SESSION_LIMIT_REACHED` and
//! `SHELL_START_FAILED` already work.
//!
//! `0x0A`..`0x10` (`TransferManifest`..`TransferResult`) bring the note
//! transfer doc §8.4/8.6/10 already implements in `transfer.rs`'s
//! `TransferSession` onto the wire, for the same reason `fsList` rides this
//! connection instead of `ALPN_TRANSFER`: that ALPN would face the exact
//! same `ControllerGate` single-connection admission problem, and is why
//! `serve.rs` has closed it with `PROTOCOL_ERROR` unconditionally since
//! phase A rather than gating it. This also departs from doc §8.3's
//! `iroh-blobs` design (content-addressed, BLAKE3-verified pull) in favor
//! of `TransferSession`'s already-built-and-tested chunk-and-credit model
//! (closer to V1's own protocol than the doc's target architecture): as of
//! this writing `iroh-blobs` is not a dependency of this crate and the dev
//! plan's own risk register still lists it as needing pre-research that
//! never happened, so building on what already exists and is proven ships
//! now instead of blocking on an unverified dependency. A future migration
//! to `iroh-blobs` can replace this frame set without touching
//! `TransferSession`'s validation/writing logic, which is transport-agnostic
//! by construction.
//!
//! `TransferChunk` is, like `Data`, not JSON: its payload is
//! `varint(fileIndex) + varint(offset) + raw bytes`, kept out of JSON so a
//! multi-hundred-KB chunk is not re-encoded as an escaped string on every
//! send.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AgentError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenPayload {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenedPayload {
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    pub shell: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResizePayload {
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellEventPayload {
    pub event: String,
    /// "osc133" | "osc633" - which integration emitted the event. Optional so
    /// a future cwd-only event source does not have to invent one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "exitCode")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsEntry {
    pub name: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsListPayload {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsListResultPayload {
    pub entries: Vec<FsEntry>,
}

/// "created" | "deleted" | "renamed" | "unknown" - candidate doc §4.7: a
/// coarse "something changed under this path" signal, not a diff. The
/// agent's poll-based watch (§fs_browse) cannot always tell which of the
/// three happened without racing the filesystem, so "unknown" is a normal,
/// expected value here, same as the local Node `fs.watch` source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsChangedPayload {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosePayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Shell exit status when `reason` is `shell_exited`; the plugin's
    /// `TerminalExitEvent` surfaces it to the UI, matching V1's
    /// `terminal.close` payload.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "exitCode")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferEntry {
    pub index: usize,
    #[serde(rename = "relativePath")]
    pub relative_path: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferManifestPayload {
    #[serde(rename = "transferId")]
    pub transfer_id: String,
    #[serde(rename = "rootNote")]
    pub root_note: String,
    pub entries: Vec<TransferEntry>,
    /// Doc §7.6: when this names a session with a known cwd, that cwd is
    /// the landing directory instead of `receive_root`.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionId")]
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferAcceptedPayload {
    #[serde(rename = "grantedBytes")]
    pub granted_bytes: u64,
}

/// Not JSON - see the module doc. `file_index` and `offset` are encoded as
/// varints (the same helper `FrameDecoder` itself uses), immediately
/// followed by the raw chunk bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct TransferChunkPayload {
    pub file_index: usize,
    pub offset: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferFileEndPayload {
    #[serde(rename = "fileIndex")]
    pub file_index: usize,
    #[serde(rename = "sentSize")]
    pub sent_size: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferCreditPayload {
    #[serde(rename = "grantedBytes")]
    pub granted_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferCompletePayload {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferResultPayload {
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
}

/// Handshake for the reverse direction (candidate doc phase 2B: copying a
/// directory-tree entry into the vault): the agent is the sender this time.
/// `path` is not root-confined, matching `fsList` (doc §6.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferPullRequestPayload {
    pub path: String,
}

/// Agent -> client, in reply to `TransferPullRequest`: what it is about to
/// send (a single file is one entry named by its own basename; a directory
/// is walked recursively). Reuses `TransferEntry` - same shape, only the
/// direction of the stream it travels on differs. After this, the client
/// grants credit with the same `TransferCredit` frame the agent normally
/// receives, and the agent sends `TransferChunk`/`TransferFileEnd` for each
/// entry and a final `TransferResult`, all frame kinds already defined for
/// the push direction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferPullManifestPayload {
    pub entries: Vec<TransferEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Open(OpenPayload),
    Opened(OpenedPayload),
    Error(ErrorPayload),
    /// Raw PTY bytes, either direction. The only frame whose payload is not
    /// JSON - wrapping terminal output in JSON would mean escaping every
    /// byte and re-encoding on every keystroke for no benefit.
    Data(Vec<u8>),
    Resize(ResizePayload),
    ShellEvent(ShellEventPayload),
    Close(ClosePayload),
    /// Handshake frame for a directory-tree stream, mutually exclusive with
    /// `Open` as the first frame on a bi-stream (see the module doc).
    FsList(FsListPayload),
    FsListResult(FsListResultPayload),
    /// Sent after `FsListResult`, zero or more times, until the stream
    /// closes (closing the stream is how the client "unwatches").
    FsChanged(FsChangedPayload),
    /// Handshake frame for a note-transfer stream, mutually exclusive with
    /// `Open`/`FsList` as the first frame on a bi-stream (see the module doc).
    TransferManifest(TransferManifestPayload),
    TransferAccepted(TransferAcceptedPayload),
    TransferChunk(TransferChunkPayload),
    TransferFileEnd(TransferFileEndPayload),
    TransferCredit(TransferCreditPayload),
    TransferComplete(TransferCompletePayload),
    TransferResult(TransferResultPayload),
    /// Handshake frame for a pull stream (candidate doc phase 2B), mutually
    /// exclusive with `Open`/`FsList`/`TransferManifest` as the first frame.
    TransferPullRequest(TransferPullRequestPayload),
    TransferPullManifest(TransferPullManifestPayload),
}

const KIND_OPEN: u8 = 0x00;
const KIND_DATA: u8 = 0x01;
const KIND_RESIZE: u8 = 0x02;
const KIND_SHELL_EVENT: u8 = 0x03;
const KIND_CLOSE: u8 = 0x04;
const KIND_OPENED: u8 = 0x05;
const KIND_ERROR: u8 = 0x06;
const KIND_FS_LIST: u8 = 0x07;
const KIND_FS_LIST_RESULT: u8 = 0x08;
const KIND_FS_CHANGED: u8 = 0x09;
const KIND_TRANSFER_MANIFEST: u8 = 0x0A;
const KIND_TRANSFER_ACCEPTED: u8 = 0x0B;
const KIND_TRANSFER_CHUNK: u8 = 0x0C;
const KIND_TRANSFER_FILE_END: u8 = 0x0D;
const KIND_TRANSFER_CREDIT: u8 = 0x0E;
const KIND_TRANSFER_COMPLETE: u8 = 0x0F;
const KIND_TRANSFER_RESULT: u8 = 0x10;
const KIND_TRANSFER_PULL_REQUEST: u8 = 0x11;
const KIND_TRANSFER_PULL_MANIFEST: u8 = 0x12;

/// A corrupt or hostile peer could otherwise claim an arbitrarily large
/// length prefix and make the agent allocate an unbounded buffer before a
/// single byte of the payload has even arrived. 1 MiB comfortably covers a
/// full terminal screen's worth of output in one frame.
const MAX_FRAME_LEN: u64 = 1024 * 1024;

impl Frame {
    fn kind(&self) -> u8 {
        match self {
            Frame::Open(_) => KIND_OPEN,
            Frame::Data(_) => KIND_DATA,
            Frame::Resize(_) => KIND_RESIZE,
            Frame::ShellEvent(_) => KIND_SHELL_EVENT,
            Frame::Close(_) => KIND_CLOSE,
            Frame::Opened(_) => KIND_OPENED,
            Frame::Error(_) => KIND_ERROR,
            Frame::FsList(_) => KIND_FS_LIST,
            Frame::FsListResult(_) => KIND_FS_LIST_RESULT,
            Frame::FsChanged(_) => KIND_FS_CHANGED,
            Frame::TransferManifest(_) => KIND_TRANSFER_MANIFEST,
            Frame::TransferAccepted(_) => KIND_TRANSFER_ACCEPTED,
            Frame::TransferChunk(_) => KIND_TRANSFER_CHUNK,
            Frame::TransferFileEnd(_) => KIND_TRANSFER_FILE_END,
            Frame::TransferCredit(_) => KIND_TRANSFER_CREDIT,
            Frame::TransferComplete(_) => KIND_TRANSFER_COMPLETE,
            Frame::TransferResult(_) => KIND_TRANSFER_RESULT,
            Frame::TransferPullRequest(_) => KIND_TRANSFER_PULL_REQUEST,
            Frame::TransferPullManifest(_) => KIND_TRANSFER_PULL_MANIFEST,
        }
    }

    fn payload_bytes(&self) -> Result<Vec<u8>, AgentError> {
        Ok(match self {
            Frame::Data(bytes) => bytes.clone(),
            Frame::Open(p) => encode_json(p)?,
            Frame::Opened(p) => encode_json(p)?,
            Frame::Error(p) => encode_json(p)?,
            Frame::Resize(p) => encode_json(p)?,
            Frame::ShellEvent(p) => encode_json(p)?,
            Frame::Close(p) => encode_json(p)?,
            Frame::FsList(p) => encode_json(p)?,
            Frame::FsListResult(p) => encode_json(p)?,
            Frame::FsChanged(p) => encode_json(p)?,
            Frame::TransferManifest(p) => encode_json(p)?,
            Frame::TransferAccepted(p) => encode_json(p)?,
            Frame::TransferChunk(p) => encode_transfer_chunk(p),
            Frame::TransferFileEnd(p) => encode_json(p)?,
            Frame::TransferCredit(p) => encode_json(p)?,
            Frame::TransferComplete(p) => encode_json(p)?,
            Frame::TransferResult(p) => encode_json(p)?,
            Frame::TransferPullRequest(p) => encode_json(p)?,
            Frame::TransferPullManifest(p) => encode_json(p)?,
        })
    }

    /// Serialises this frame to the bytes that should be written to the
    /// stream. Does not itself touch any transport.
    pub fn encode(&self) -> Result<Vec<u8>, AgentError> {
        let payload = self.payload_bytes()?;
        let mut out = Vec::with_capacity(1 + 10 + payload.len());
        out.push(self.kind());
        write_varint(&mut out, payload.len() as u64);
        out.extend_from_slice(&payload);
        Ok(out)
    }

    fn from_kind_and_payload(kind: u8, payload: Vec<u8>) -> Result<Self, AgentError> {
        Ok(match kind {
            KIND_OPEN => Frame::Open(decode_json(&payload)?),
            KIND_DATA => Frame::Data(payload),
            KIND_RESIZE => Frame::Resize(decode_json(&payload)?),
            KIND_SHELL_EVENT => Frame::ShellEvent(decode_json(&payload)?),
            KIND_CLOSE => Frame::Close(decode_json(&payload)?),
            KIND_OPENED => Frame::Opened(decode_json(&payload)?),
            KIND_ERROR => Frame::Error(decode_json(&payload)?),
            KIND_FS_LIST => Frame::FsList(decode_json(&payload)?),
            KIND_FS_LIST_RESULT => Frame::FsListResult(decode_json(&payload)?),
            KIND_FS_CHANGED => Frame::FsChanged(decode_json(&payload)?),
            KIND_TRANSFER_MANIFEST => Frame::TransferManifest(decode_json(&payload)?),
            KIND_TRANSFER_ACCEPTED => Frame::TransferAccepted(decode_json(&payload)?),
            KIND_TRANSFER_CHUNK => Frame::TransferChunk(decode_transfer_chunk(&payload)?),
            KIND_TRANSFER_FILE_END => Frame::TransferFileEnd(decode_json(&payload)?),
            KIND_TRANSFER_CREDIT => Frame::TransferCredit(decode_json(&payload)?),
            KIND_TRANSFER_COMPLETE => Frame::TransferComplete(decode_json(&payload)?),
            KIND_TRANSFER_RESULT => Frame::TransferResult(decode_json(&payload)?),
            KIND_TRANSFER_PULL_REQUEST => Frame::TransferPullRequest(decode_json(&payload)?),
            KIND_TRANSFER_PULL_MANIFEST => Frame::TransferPullManifest(decode_json(&payload)?),
            other => {
                return Err(AgentError::Protocol(format!(
                    "unknown termy/terminal/1 frame kind {other:#04x}"
                )))
            }
        })
    }
}

fn encode_json<T: Serialize>(value: &T) -> Result<Vec<u8>, AgentError> {
    serde_json::to_vec(value)
        .map_err(|e| AgentError::Protocol(format!("cannot encode frame payload: {e}")))
}

fn decode_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, AgentError> {
    serde_json::from_slice(bytes)
        .map_err(|e| AgentError::Protocol(format!("cannot decode frame payload: {e}")))
}

/// `varint(file_index) + varint(offset) + raw bytes` (see the module doc).
fn encode_transfer_chunk(payload: &TransferChunkPayload) -> Vec<u8> {
    let mut out = Vec::with_capacity(20 + payload.data.len());
    write_varint(&mut out, payload.file_index as u64);
    write_varint(&mut out, payload.offset);
    out.extend_from_slice(&payload.data);
    out
}

fn decode_transfer_chunk(bytes: &[u8]) -> Result<TransferChunkPayload, AgentError> {
    let (file_index, consumed1) = try_read_varint(bytes)?.ok_or_else(|| {
        AgentError::Protocol("transferChunk payload truncated (fileIndex)".into())
    })?;
    let (offset, consumed2) = try_read_varint(&bytes[consumed1..])?
        .ok_or_else(|| AgentError::Protocol("transferChunk payload truncated (offset)".into()))?;
    Ok(TransferChunkPayload {
        file_index: file_index as usize,
        offset,
        data: bytes[consumed1 + consumed2..].to_vec(),
    })
}

/// LEB128 unsigned varint, as used by protobuf. QUIC streams are byte
/// streams, not message-framed, so something has to mark where one frame's
/// payload ends and the next frame's kind byte begins.
fn write_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

/// Returns `Some((value, bytes_consumed))` if `bytes` starts with a
/// complete varint, `None` if it might still be incomplete (more bytes
/// needed). A run of ten continuation bytes cannot happen for any value
/// that would pass the `MAX_FRAME_LEN` check below, so that case is treated
/// as a protocol violation rather than "wait for more data".
fn try_read_varint(bytes: &[u8]) -> Result<Option<(u64, usize)>, AgentError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if i >= 10 {
            return Err(AgentError::Protocol(
                "termy/terminal/1 frame length prefix is too long".into(),
            ));
        }
        value |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(Some((value, i + 1)));
        }
        shift += 7;
    }
    Ok(None)
}

/// Accumulates bytes arriving from a `termy/terminal/1` stream and pops
/// complete frames off the front. Kept transport-agnostic on purpose: feed
/// it whatever chunks the underlying reader hands back, in whatever sizes
/// they happen to arrive in.
#[derive(Debug, Default)]
pub struct FrameDecoder {
    buf: Vec<u8>,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Pops one frame if the buffer already holds a complete one. `Ok(None)`
    /// means "not enough bytes yet" and is not an error - the caller should
    /// read more from the stream and call this again.
    pub fn next_frame(&mut self) -> Result<Option<Frame>, AgentError> {
        if self.buf.is_empty() {
            return Ok(None);
        }

        let kind = self.buf[0];
        let Some((len, len_bytes)) = try_read_varint(&self.buf[1..])? else {
            return Ok(None);
        };
        if len > MAX_FRAME_LEN {
            return Err(AgentError::Protocol(format!(
                "termy/terminal/1 frame of {len} bytes exceeds the {MAX_FRAME_LEN}-byte limit"
            )));
        }

        let header_len = 1 + len_bytes;
        let total_len = header_len + len as usize;
        if self.buf.len() < total_len {
            return Ok(None);
        }

        let payload = self.buf[header_len..total_len].to_vec();
        self.buf.drain(..total_len);
        Frame::from_kind_and_payload(kind, payload).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(frame: Frame) {
        let encoded = frame.encode().unwrap();
        let mut decoder = FrameDecoder::new();
        decoder.push(&encoded);
        assert_eq!(decoder.next_frame().unwrap(), Some(frame));
        assert_eq!(decoder.next_frame().unwrap(), None, "buffer must be drained");
    }

    #[test]
    fn every_frame_kind_round_trips() {
        roundtrip(Frame::Open(OpenPayload { cols: 80, rows: 24 }));
        roundtrip(Frame::Opened(OpenedPayload {
            session_id: Uuid::new_v4(),
            shell: "/bin/bash".into(),
        }));
        roundtrip(Frame::Error(ErrorPayload {
            message: "SESSION_LIMIT_REACHED".into(),
        }));
        roundtrip(Frame::Data(b"echo hi\n".to_vec()));
        roundtrip(Frame::Data(Vec::new()));
        roundtrip(Frame::Resize(ResizePayload { cols: 120, rows: 40 }));
        roundtrip(Frame::ShellEvent(ShellEventPayload {
            event: "command_end".into(),
            source: Some("osc133".into()),
            cwd: Some("/home/user/project".into()),
            exit_code: Some(0),
        }));
        roundtrip(Frame::ShellEvent(ShellEventPayload {
            event: "prompt_start".into(),
            source: None,
            cwd: None,
            exit_code: None,
        }));
        roundtrip(Frame::Close(ClosePayload {
            reason: Some("peer disconnected".into()),
            exit_code: None,
        }));
        roundtrip(Frame::Close(ClosePayload {
            reason: Some("shell_exited".into()),
            exit_code: Some(0),
        }));
        roundtrip(Frame::Close(ClosePayload {
            reason: None,
            exit_code: None,
        }));
        roundtrip(Frame::FsList(FsListPayload {
            path: "/home/user/project".into(),
        }));
        roundtrip(Frame::FsListResult(FsListResultPayload {
            entries: vec![
                FsEntry {
                    name: "src".into(),
                    is_directory: true,
                },
                FsEntry {
                    name: "readme.md".into(),
                    is_directory: false,
                },
            ],
        }));
        roundtrip(Frame::FsListResult(FsListResultPayload { entries: vec![] }));
        roundtrip(Frame::FsChanged(FsChangedPayload {
            kind: "unknown".into(),
        }));
        roundtrip(Frame::TransferManifest(TransferManifestPayload {
            transfer_id: "transfer-1".into(),
            root_note: "notes/demo.md".into(),
            entries: vec![
                TransferEntry {
                    index: 0,
                    relative_path: "notes/demo.md".into(),
                    size: 11,
                },
                TransferEntry {
                    index: 1,
                    relative_path: "assets/img.png".into(),
                    size: 0,
                },
            ],
            session_id: None,
        }));
        roundtrip(Frame::TransferManifest(TransferManifestPayload {
            transfer_id: "transfer-2".into(),
            root_note: "a.md".into(),
            entries: vec![TransferEntry {
                index: 0,
                relative_path: "a.md".into(),
                size: 1,
            }],
            session_id: Some(Uuid::nil()),
        }));
        roundtrip(Frame::TransferAccepted(TransferAcceptedPayload {
            granted_bytes: 4 * 1024 * 1024,
        }));
        roundtrip(Frame::TransferChunk(TransferChunkPayload {
            file_index: 0,
            offset: 0,
            data: b"hello world".to_vec(),
        }));
        roundtrip(Frame::TransferChunk(TransferChunkPayload {
            file_index: 3,
            offset: 300_000,
            data: Vec::new(),
        }));
        roundtrip(Frame::TransferFileEnd(TransferFileEndPayload {
            file_index: 0,
            sent_size: 11,
        }));
        roundtrip(Frame::TransferCredit(TransferCreditPayload {
            granted_bytes: 8 * 1024 * 1024,
        }));
        roundtrip(Frame::TransferComplete(TransferCompletePayload {}));
        roundtrip(Frame::TransferResult(TransferResultPayload {
            success: true,
            code: None,
            message: String::new(),
        }));
        roundtrip(Frame::TransferResult(TransferResultPayload {
            success: false,
            code: Some("WRITE_FAILED".into()),
            message: "disk full".into(),
        }));
        roundtrip(Frame::TransferPullRequest(TransferPullRequestPayload {
            path: "/home/user/project/notes".into(),
        }));
        roundtrip(Frame::TransferPullManifest(TransferPullManifestPayload {
            entries: vec![
                TransferEntry {
                    index: 0,
                    relative_path: "notes/demo.md".into(),
                    size: 11,
                },
                TransferEntry {
                    index: 1,
                    relative_path: "notes/assets/img.png".into(),
                    size: 0,
                },
            ],
        }));
        roundtrip(Frame::TransferPullManifest(TransferPullManifestPayload {
            entries: vec![],
        }));
    }

    #[test]
    fn a_transfer_chunk_with_a_large_offset_round_trips_its_varint() {
        roundtrip(Frame::TransferChunk(TransferChunkPayload {
            file_index: 1,
            offset: (MAX_FRAME_LEN - 20) * 3,
            data: vec![7u8; 64],
        }));
    }

    #[test]
    fn two_frames_back_to_back_decode_in_order() {
        let first = Frame::Data(b"one".to_vec());
        let second = Frame::Data(b"two".to_vec());

        let mut decoder = FrameDecoder::new();
        decoder.push(&first.encode().unwrap());
        decoder.push(&second.encode().unwrap());

        assert_eq!(decoder.next_frame().unwrap(), Some(first));
        assert_eq!(decoder.next_frame().unwrap(), Some(second));
        assert_eq!(decoder.next_frame().unwrap(), None);
    }

    #[test]
    fn a_frame_split_across_many_small_chunks_still_decodes() {
        let frame = Frame::ShellEvent(ShellEventPayload {
            event: "command_end".into(),
            source: Some("osc633".into()),
            cwd: Some("/tmp/some/fairly/long/path/for/varint/coverage".into()),
            exit_code: Some(1),
        });
        let encoded = frame.encode().unwrap();

        let mut decoder = FrameDecoder::new();
        for byte in &encoded {
            assert_eq!(decoder.next_frame().unwrap(), None, "must not decode early");
            decoder.push(std::slice::from_ref(byte));
        }
        assert_eq!(decoder.next_frame().unwrap(), Some(frame));
    }

    #[test]
    fn an_unknown_kind_byte_is_a_protocol_error() {
        let mut decoder = FrameDecoder::new();
        decoder.push(&[0x7f, 0x00]); // kind 0x7f, zero-length payload
        assert!(matches!(
            decoder.next_frame(),
            Err(AgentError::Protocol(_))
        ));
    }

    #[test]
    fn a_length_prefix_over_the_cap_is_rejected_without_buffering_the_payload() {
        let mut decoder = FrameDecoder::new();
        let mut out = vec![KIND_DATA];
        write_varint(&mut out, MAX_FRAME_LEN + 1);
        decoder.push(&out);
        assert!(matches!(
            decoder.next_frame(),
            Err(AgentError::Protocol(_))
        ));
    }

    #[test]
    fn malformed_json_in_a_structured_frame_is_a_protocol_error() {
        let mut decoder = FrameDecoder::new();
        let mut out = vec![KIND_RESIZE];
        write_varint(&mut out, 2);
        out.extend_from_slice(b"{}"); // valid JSON, but missing required fields
        decoder.push(&out);
        assert!(matches!(
            decoder.next_frame(),
            Err(AgentError::Protocol(_))
        ));
    }

    #[test]
    fn varint_round_trips_across_the_single_and_multi_byte_boundary() {
        for value in [0u64, 1, 127, 128, 300, 16384, MAX_FRAME_LEN] {
            let mut out = Vec::new();
            write_varint(&mut out, value);
            assert_eq!(try_read_varint(&out).unwrap(), Some((value, out.len())));
        }
    }
}
