//! Unfold core: everything the viewer needs that is not tied to a GUI framework.
//!
//! This crate must not depend on any UI library. The GPUI app at the repository
//! root turns these plain types into pixels; a CLI could reuse them too.
//!
//! - [`parser`]: the flat-array JSON tree and the builder that fills it
//! - [`flat_row`]: visible rows for virtual scrolling, built from the tree
//! - [`search`]: text and regex search over the tree
//! - [`json_export`]: copy and export of a node as JSON
//! - [`config`]: persisted user settings
//! - [`parse_error`]: readable JSON parse errors

pub mod config;
pub mod flat_row;
pub mod json_export;
pub mod parse_error;
pub mod parser;
pub mod search;
