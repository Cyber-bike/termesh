//! Runtime schema validation, sharing one bundled schema with the Node side.
//!
//! Why this exists on top of the generated structs: typify expresses only part
//! of the schema in the type system. `minimum: 1` becomes `NonZeroU64`, but
//! `maximum: 1000` is dropped entirely, so `serde_json::from_str::<ControlMessage>`
//! happily accepts `cols: 99999` while the Node validator rejects it. Rather
//! than hand-maintain a parallel list of bounds - which would drift from the
//! schema the moment anyone edits it - both ends run the same document.
//!
//! Call [`validate`] before trusting a control message. Cross-field rules that
//! JSON Schema cannot express (index continuity, rootNote match, code/success
//! agreement) still live outside this module; see `protocol/tools/semantic.js`.

use std::sync::OnceLock;

use jsonschema::Validator;

/// The bundle generated from `protocol/schema/`. Embedded so the binary carries
/// its own contract and cannot fall out of step with the crate it ships in.
pub const BUNDLED_SCHEMA: &str = include_str!("../../protocol.bundle.schema.json");

fn validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let schema: serde_json::Value =
            serde_json::from_str(BUNDLED_SCHEMA).expect("bundled schema is not valid JSON");
        jsonschema::options()
            .build(&schema)
            .expect("bundled schema is not a valid Draft 2020-12 schema")
    })
}

/// Validates a decoded control message against the protocol schema.
///
/// Returns every violation rather than just the first, so a rejection can be
/// logged in one line without re-running the check.
pub fn validate(instance: &serde_json::Value) -> Result<(), Vec<String>> {
    let errors: Vec<String> = validator()
        .iter_errors(instance)
        .map(|e| format!("{}: {}", e.instance_path(), e))
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Convenience wrapper: parse and validate in one step.
///
/// Validation runs before deserialization so the caller gets the schema's
/// message ("1001 is greater than the maximum of 1000") instead of serde's
/// less specific shape error.
pub fn parse_validated(raw: &str) -> Result<crate::messages::ControlMessage, Vec<String>> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| vec![format!("invalid JSON: {e}")])?;
    validate(&value)?;
    serde_json::from_value(value).map_err(|e| vec![format!("decode failed: {e}")])
}
