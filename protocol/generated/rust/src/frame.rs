//! Binary frame codec (doc 8.5). Hand-written, not generated.
//!
//! Mirrors `protocol/tools/frame-codec.js`; both are checked against the shared
//! hex vectors in `protocol/fixtures/frames/`, which is what keeps the two
//! implementations honest.

use std::collections::HashMap;

pub const HEADER_BYTES: usize = 38;
pub const MAGIC: [u8; 2] = [0x54, 0x4d]; // "TM"
pub const VERSION: u8 = 0x01;

pub const KIND_TERMINAL_INPUT: u8 = 0x01;
pub const KIND_TERMINAL_OUTPUT: u8 = 0x02;
pub const KIND_FILE_CHUNK: u8 = 0x03;

pub const TERMINAL_PAYLOAD_MAX: usize = 32 * 1024;
pub const FILE_PAYLOAD_MAX: usize = 256 * 1024;
pub const MESSAGE_MAX: usize = HEADER_BYTES + FILE_PAYLOAD_MAX; // 262182
pub const TERMINAL_FILE_INDEX: u32 = 0xffff_ffff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameError(pub String);

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for FrameError {}

fn err<T>(msg: impl Into<String>) -> Result<T, FrameError> {
    Err(FrameError(msg.into()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub kind: u8,
    /// sessionId for terminal frames, transferId for file chunks.
    pub stream_id: [u8; 16],
    pub file_index: u32,
    pub offset: u64,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn stream_uuid(&self) -> String {
        let h: String = self.stream_id.iter().map(|b| format!("{b:02x}")).collect();
        format!(
            "{}-{}-{}-{}-{}",
            &h[0..8],
            &h[8..12],
            &h[12..16],
            &h[16..20],
            &h[20..32]
        )
    }

    pub fn is_file(&self) -> bool {
        self.kind == KIND_FILE_CHUNK
    }
}

pub fn payload_limit(kind: u8) -> usize {
    if kind == KIND_FILE_CHUNK {
        FILE_PAYLOAD_MAX
    } else {
        TERMINAL_PAYLOAD_MAX
    }
}

pub fn encode(frame: &Frame) -> Result<Vec<u8>, FrameError> {
    if !matches!(
        frame.kind,
        KIND_TERMINAL_INPUT | KIND_TERMINAL_OUTPUT | KIND_FILE_CHUNK
    ) {
        return err(format!("unknown kind 0x{:x}", frame.kind));
    }
    if frame.payload.len() > payload_limit(frame.kind) {
        return err(format!(
            "payload {} exceeds the limit for kind 0x{:x}",
            frame.payload.len(),
            frame.kind
        ));
    }

    let file_index = if frame.kind == KIND_FILE_CHUNK {
        frame.file_index
    } else {
        TERMINAL_FILE_INDEX
    };

    let mut out = Vec::with_capacity(HEADER_BYTES + frame.payload.len());
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.push(frame.kind);
    out.push(0); // flags
    out.push(0); // reserved
    out.extend_from_slice(&(frame.payload.len() as u32).to_be_bytes());
    out.extend_from_slice(&frame.stream_id);
    out.extend_from_slice(&file_index.to_be_bytes());
    out.extend_from_slice(&frame.offset.to_be_bytes());
    out.extend_from_slice(&frame.payload);
    Ok(out)
}

pub fn decode(buf: &[u8]) -> Result<Frame, FrameError> {
    if buf.len() > MESSAGE_MAX {
        return err(format!("message {} exceeds {MESSAGE_MAX}", buf.len()));
    }
    if buf.len() < HEADER_BYTES {
        return err(format!("truncated header: {} < {HEADER_BYTES}", buf.len()));
    }
    if buf[0..2] != MAGIC {
        return err("bad magic");
    }
    if buf[2] != VERSION {
        return err(format!("unsupported frame version {}", buf[2]));
    }

    let kind = buf[3];
    if !matches!(
        kind,
        KIND_TERMINAL_INPUT | KIND_TERMINAL_OUTPUT | KIND_FILE_CHUNK
    ) {
        return err(format!("unknown kind 0x{kind:x}"));
    }
    if buf[4] != 0 {
        return err("flags must be 0 in MVP");
    }
    if buf[5] != 0 {
        return err("reserved byte must be 0");
    }

    let payload_length = u32::from_be_bytes([buf[6], buf[7], buf[8], buf[9]]) as usize;
    let actual = buf.len() - HEADER_BYTES;
    if payload_length != actual {
        return err(format!(
            "payloadLength {payload_length} does not match the {actual} bytes present"
        ));
    }
    if payload_length > payload_limit(kind) {
        return err(format!(
            "payload {payload_length} exceeds the limit for kind 0x{kind:x}"
        ));
    }

    let mut stream_id = [0u8; 16];
    stream_id.copy_from_slice(&buf[10..26]);

    let file_index = u32::from_be_bytes([buf[26], buf[27], buf[28], buf[29]]);
    if kind == KIND_FILE_CHUNK {
        if file_index > 255 {
            return err(format!(
                "fileIndex {file_index} exceeds the 256-file batch limit"
            ));
        }
    } else if file_index != TERMINAL_FILE_INDEX {
        return err("terminal frames must set fileIndex to 0xFFFFFFFF");
    }

    let offset = u64::from_be_bytes([
        buf[30], buf[31], buf[32], buf[33], buf[34], buf[35], buf[36], buf[37],
    ]);

    Ok(Frame {
        kind,
        stream_id,
        file_index,
        offset,
        payload: buf[HEADER_BYTES..].to_vec(),
    })
}

/// Per-counting-domain offset continuity (doc 8.5).
///
/// File frames key on `(transferId, fileIndex)` and a gap is fatal; terminal
/// frames key on `(sessionId, kind)` and a gap is only reported, because WSS
/// already guarantees ordering and killing a live terminal over a bookkeeping
/// mismatch costs the user more than it protects.
#[derive(Default)]
pub struct OffsetTracker {
    domains: HashMap<([u8; 16], u32, u8), u64>,
}

pub struct OffsetCheck {
    pub ok: bool,
    pub fatal: bool,
    pub expected: u64,
    pub got: u64,
}

impl OffsetTracker {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(frame: &Frame) -> ([u8; 16], u32, u8) {
        if frame.is_file() {
            (frame.stream_id, frame.file_index, KIND_FILE_CHUNK)
        } else {
            (frame.stream_id, 0, frame.kind)
        }
    }

    pub fn accept(&mut self, frame: &Frame) -> OffsetCheck {
        let key = Self::key(frame);
        let expected = *self.domains.get(&key).unwrap_or(&0);
        let ok = expected == frame.offset;
        if ok {
            self.domains
                .insert(key, expected + frame.payload.len() as u64);
        }
        OffsetCheck {
            ok,
            fatal: !ok && frame.is_file(),
            expected,
            got: frame.offset,
        }
    }

    pub fn forget_stream(&mut self, stream_id: &[u8; 16]) {
        self.domains.retain(|(id, _, _), _| id != stream_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: [u8; 16] = [1u8; 16];
    const TRANSFER: [u8; 16] = [2u8; 16];

    fn terminal(offset: u64, len: usize) -> Frame {
        Frame {
            kind: KIND_TERMINAL_OUTPUT,
            stream_id: SESSION,
            file_index: TERMINAL_FILE_INDEX,
            offset,
            payload: vec![0xab; len],
        }
    }

    #[test]
    fn round_trip() {
        let frame = terminal(0, 7);
        let encoded = encode(&frame).unwrap();
        assert_eq!(encoded.len(), HEADER_BYTES + 7);
        assert_eq!(decode(&encoded).unwrap(), frame);
    }

    #[test]
    fn max_file_chunk_matches_the_documented_ceiling() {
        let frame = Frame {
            kind: KIND_FILE_CHUNK,
            stream_id: TRANSFER,
            file_index: 0,
            offset: 0,
            payload: vec![0x5a; FILE_PAYLOAD_MAX],
        };
        assert_eq!(encode(&frame).unwrap().len(), 262_182);
    }

    #[test]
    fn rejects_malformed_headers() {
        let good = encode(&terminal(0, 3)).unwrap();

        let mut bad_magic = good.clone();
        bad_magic[0] = 0x58;
        assert!(decode(&bad_magic).unwrap_err().0.contains("bad magic"));

        let mut bad_version = good.clone();
        bad_version[2] = 0x02;
        assert!(decode(&bad_version)
            .unwrap_err()
            .0
            .contains("unsupported frame version"));

        let mut bad_kind = good.clone();
        bad_kind[3] = 0x09;
        assert!(decode(&bad_kind).unwrap_err().0.contains("unknown kind"));

        let mut flags = good.clone();
        flags[4] = 1;
        assert!(decode(&flags).unwrap_err().0.contains("flags must be 0"));

        let mut reserved = good.clone();
        reserved[5] = 1;
        assert!(decode(&reserved)
            .unwrap_err()
            .0
            .contains("reserved byte must be 0"));

        let mut length = good.clone();
        length[6..10].copy_from_slice(&999u32.to_be_bytes());
        assert!(decode(&length).unwrap_err().0.contains("does not match"));

        let mut file_index = good.clone();
        file_index[26..30].copy_from_slice(&3u32.to_be_bytes());
        assert!(decode(&file_index)
            .unwrap_err()
            .0
            .contains("terminal frames must set fileIndex"));

        assert!(decode(&good[..20])
            .unwrap_err()
            .0
            .contains("truncated header"));
    }

    #[test]
    fn offsets_are_tracked_per_domain() {
        let mut tracker = OffsetTracker::new();

        assert!(tracker.accept(&terminal(0, 100)).ok);
        // Input counts separately from output even though the session matches.
        let input = Frame {
            kind: KIND_TERMINAL_INPUT,
            ..terminal(0, 10)
        };
        assert!(tracker.accept(&input).ok);
        assert!(tracker.accept(&terminal(100, 50)).ok);

        // Files count per fileIndex within one transfer.
        let file = |idx: u32, offset: u64, len: usize| Frame {
            kind: KIND_FILE_CHUNK,
            stream_id: TRANSFER,
            file_index: idx,
            offset,
            payload: vec![7; len],
        };
        assert!(tracker.accept(&file(0, 0, 256)).ok);
        assert!(tracker.accept(&file(1, 0, 128)).ok);
        assert!(tracker.accept(&file(0, 256, 64)).ok);
    }

    #[test]
    fn a_file_gap_is_fatal_but_a_terminal_gap_is_not() {
        let mut tracker = OffsetTracker::new();

        let file_gap = Frame {
            kind: KIND_FILE_CHUNK,
            stream_id: TRANSFER,
            file_index: 0,
            offset: 64,
            payload: vec![1; 8],
        };
        let check = tracker.accept(&file_gap);
        assert!(!check.ok && check.fatal);

        let check = tracker.accept(&terminal(64, 8));
        assert!(!check.ok && !check.fatal);
    }
}
