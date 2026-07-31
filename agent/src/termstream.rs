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
pub struct ClosePayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Shell exit status when `reason` is `shell_exited`; the plugin's
    /// `TerminalExitEvent` surfaces it to the UI, matching V1's
    /// `terminal.close` payload.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "exitCode")]
    pub exit_code: Option<i32>,
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
}

const KIND_OPEN: u8 = 0x00;
const KIND_DATA: u8 = 0x01;
const KIND_RESIZE: u8 = 0x02;
const KIND_SHELL_EVENT: u8 = 0x03;
const KIND_CLOSE: u8 = 0x04;
const KIND_OPENED: u8 = 0x05;
const KIND_ERROR: u8 = 0x06;

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
