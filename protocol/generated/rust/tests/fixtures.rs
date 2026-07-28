//! Cross-end contract test: the Rust decoder must agree with the Node validator
//! on the shared fixture suite (doc 8.1).
//!
//! The two ends do not have identical reach, and the split is deliberate:
//! serde enforces everything the schema expresses (types, enums, ranges via
//! newtypes, patterns, `deny_unknown_fields`), while the cross-field rules in
//! `protocol/tools/semantic.js` are not expressible in JSON Schema and so cannot
//! be enforced by a derived deserializer either. Those fixtures are listed in
//! `SEMANTIC_ONLY` and are expected to deserialize cleanly here; whichever end
//! consumes them must run the semantic checks separately.

use std::fs;
use std::path::{Path, PathBuf};

use termy_protocol::ControlMessage;

/// Invalid fixtures that violate a cross-field rule rather than the schema, so
/// serde alone cannot reject them.
const SEMANTIC_ONLY: &[&str] = &[
    "terminal-close-shell-exited-null-exitcode.json",
    "transfer-result-success-with-code.json",
    "transfer-start-duplicate-path.json",
    "transfer-start-index-gap.json",
    "transfer-start-rootnote-mismatch.json",
];

fn fixture_dir(kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(kind)
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_valid_fixture_deserializes() {
    let dir = fixture_dir("valid");
    let files = json_files(&dir);
    assert!(
        !files.is_empty(),
        "no valid fixtures found in {}",
        dir.display()
    );

    for path in &files {
        let raw = fs::read_to_string(path).unwrap();
        let parsed: Result<ControlMessage, _> = serde_json::from_str(&raw);
        assert!(
            parsed.is_ok(),
            "{} should deserialize but failed: {}",
            path.file_name().unwrap().to_string_lossy(),
            parsed.unwrap_err()
        );
    }

    println!("{} valid fixtures deserialized", files.len());
}

#[test]
fn valid_fixtures_round_trip_without_losing_fields() {
    for path in json_files(&fixture_dir("valid")) {
        let raw = fs::read_to_string(&path).unwrap();
        let original: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let message: ControlMessage = serde_json::from_str(&raw).unwrap();
        let reencoded = serde_json::to_value(&message).unwrap();

        assert_eq!(
            original,
            reencoded,
            "{} did not survive a round trip",
            path.file_name().unwrap().to_string_lossy()
        );
    }
}

#[test]
fn schema_level_invalid_fixtures_are_rejected() {
    let dir = fixture_dir("invalid");
    let files = json_files(&dir);
    assert!(!files.is_empty(), "no invalid fixtures found");

    let mut rejected = 0;
    let mut semantic = 0;

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let raw = fs::read_to_string(path).unwrap();
        let parsed = termy_protocol::parse_validated(&raw);

        if SEMANTIC_ONLY.contains(&name.as_str()) {
            assert!(
                parsed.is_ok(),
                "{name} is listed as semantic-only but schema validation already rejects it; \
                 move it out of SEMANTIC_ONLY"
            );
            semantic += 1;
        } else {
            assert!(
                parsed.is_err(),
                "{name} should have been rejected but parsed"
            );
            rejected += 1;
        }
    }

    println!(
        "{rejected} fixtures rejected by schema validation, {semantic} deferred to semantic checks"
    );
}

/// Guards the reason `validate` exists at all. typify turns `minimum: 1` into
/// `NonZeroU64` and drops `maximum`, so serde on its own accepts an out-of-range
/// `cols` that the Node validator rejects. If a future typify release starts
/// emitting the upper bound, this test fails and the extra validation layer can
/// be reconsidered.
#[test]
fn serde_alone_misses_numeric_upper_bounds() {
    let raw =
        fs::read_to_string(fixture_dir("invalid").join("terminal-open-cols-out-of-range.json"))
            .unwrap();

    let via_serde: Result<ControlMessage, _> = serde_json::from_str(&raw);
    assert!(
        via_serde.is_ok(),
        "typify now enforces maximum; validate.rs may no longer be needed for bounds"
    );

    let via_schema = termy_protocol::parse_validated(&raw);
    assert!(
        via_schema.is_err(),
        "schema validation must reject cols=1001"
    );
}

#[test]
fn discriminant_is_enforced() {
    // A payload shaped like terminal.open but labelled agent.heartbeat must not
    // sneak through the untagged enum.
    let wrong_type = r#"{
        "protocolVersion": 1,
        "type": "agent.heartbeat",
        "requestId": "b7c1a2d3-4e5f-4a6b-8c9d-0e1f2a3b4c5d",
        "deviceId": "3d594650-3436-4c7a-9a15-9b5c3f0f4a11",
        "sessionId": null,
        "payload": { "cols": 120, "rows": 30 }
    }"#;

    let parsed: Result<ControlMessage, _> = serde_json::from_str(wrong_type);
    assert!(
        parsed.is_err(),
        "a mislabelled message must not deserialize"
    );
}

#[test]
fn protocol_version_is_pinned() {
    let wrong_version = r#"{
        "protocolVersion": 2,
        "type": "terminal.open",
        "requestId": "b7c1a2d3-4e5f-4a6b-8c9d-0e1f2a3b4c5d",
        "deviceId": "3d594650-3436-4c7a-9a15-9b5c3f0f4a11",
        "sessionId": null,
        "payload": { "cols": 120, "rows": 30 }
    }"#;

    let parsed: Result<ControlMessage, _> = serde_json::from_str(wrong_version);
    assert!(parsed.is_err(), "protocolVersion must be pinned to 1");
}
