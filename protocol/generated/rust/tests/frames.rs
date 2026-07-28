//! The Rust frame decoder must agree with the JS reference codec on the shared
//! hex vectors (doc 8.1). The vectors are produced by
//! `protocol/tools/frame-codec.test.js`, so a divergence in either direction
//! fails here.

use std::fs;
use std::path::{Path, PathBuf};

use termy_protocol::frame;

fn frames_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/frames")
}

#[derive(serde::Deserialize)]
struct VectorMeta {
    name: String,
    expect: String,
    note: String,
}

fn load_hex(name: &str) -> Vec<u8> {
    let raw = fs::read_to_string(frames_dir().join(format!("{name}.hex")))
        .unwrap_or_else(|e| panic!("cannot read vector {name}: {e}"));

    let hex: String = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("");

    (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex.trim()[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn rust_and_js_agree_on_every_frame_vector() {
    let index: Vec<VectorMeta> =
        serde_json::from_str(&fs::read_to_string(frames_dir().join("index.json")).unwrap())
            .unwrap();

    assert!(!index.is_empty(), "no frame vectors found");

    let mut accepted = 0;
    let mut rejected = 0;

    for meta in &index {
        let bytes = load_hex(&meta.name);
        let decoded = frame::decode(&bytes);

        match meta.expect.as_str() {
            "accept" => {
                assert!(
                    decoded.is_ok(),
                    "{} should decode ({}) but failed: {}",
                    meta.name,
                    meta.note,
                    decoded.unwrap_err()
                );
                accepted += 1;
            }
            "reject" => {
                assert!(
                    decoded.is_err(),
                    "{} should be rejected ({}) but decoded",
                    meta.name,
                    meta.note
                );
                rejected += 1;
            }
            other => panic!("unknown expectation {other} for {}", meta.name),
        }
    }

    println!("{accepted} vectors accepted, {rejected} rejected, matching the JS codec");
}

#[test]
fn a_valid_vector_decodes_to_the_expected_fields() {
    let bytes = load_hex("valid-file-chunk");
    let decoded = frame::decode(&bytes).unwrap();

    assert_eq!(decoded.kind, frame::KIND_FILE_CHUNK);
    assert_eq!(decoded.file_index, 7);
    // The JS side writes this vector at an offset above 2^32 specifically to
    // exercise the u64 field.
    assert_eq!(decoded.offset, 4_294_967_296);
    assert_eq!(decoded.payload.len(), 1024);
    assert_eq!(decoded.stream_uuid(), "e2f3a4b5-c6d7-4e8f-9a0b-1c2d3e4f5a6b");
}

#[test]
fn re_encoding_a_decoded_vector_reproduces_the_bytes() {
    for name in ["valid-terminal-output", "valid-file-chunk", "valid-empty-payload"] {
        let bytes = load_hex(name);
        let decoded = frame::decode(&bytes).unwrap();
        assert_eq!(frame::encode(&decoded).unwrap(), bytes, "{name} did not round trip");
    }
}
