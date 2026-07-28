//! termy-relay library surface.
//!
//! The binary is a thin CLI over these modules; everything testable lives here
//! so integration tests can drive it without going through a process boundary.

pub mod api;
pub mod auth;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod ratelimit;
