//! wasm/wasi backend for cursor queries.
//!
//! Reading the cursor position requires writing `ESC [ 6 n` and synchronously
//! reading the terminal's reply — the browser host (xterm.js) doesn't surface
//! that as a blocking read here, and neither ratatui nor INCIDENT call
//! `cursor::position()`, so this just reports `(0, 0)`.

use std::io;

/// Returns the cursor position (column, row); always `(0, 0)` under wasm.
pub fn position() -> io::Result<(u16, u16)> {
    Ok((0, 0))
}
