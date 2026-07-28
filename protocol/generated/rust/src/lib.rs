//! Generated Rust types for the Termy remote protocol.
//!
//! `messages.rs` is produced by typify from `protocol/generated/protocol.bundle.schema.json`,
//! which is itself bundled from `protocol/schema/`. Regenerate with
//! `npm run generate:rust` in `protocol/`; never edit it by hand.
//!
//! The bundler rewrites `const` into a single-element `enum` so that every
//! message gets a real discriminant type. That matters because [`ControlMessage`]
//! is `#[serde(untagged)]`: without a typed `type` field, serde would pick the
//! first variant whose shape happens to fit.

pub mod frame;
pub mod messages;
pub mod validate;

pub use messages::*;
pub use validate::{parse_validated, validate, BUNDLED_SCHEMA};
