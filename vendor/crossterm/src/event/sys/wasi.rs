//! wasm/wasi side of the `event` module.
//!
//! The byte-sequence parser (`parse_event`) is platform independent, so we
//! reuse it verbatim from the unix backend rather than duplicating it.

#[cfg(feature = "events")]
#[path = "unix/parse.rs"]
pub(crate) mod parse;
