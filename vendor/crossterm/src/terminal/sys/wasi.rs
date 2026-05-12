//! wasm/wasi backend for terminal operations.
//!
//! When crossterm runs in a browser (the game is compiled to `wasm32-wasip1`
//! and hosted by an xterm.js frontend) there is no termios and no `ioctl`:
//!  * "raw mode" is a no-op — the host already delivers raw keystroke bytes on
//!    stdin and does no line editing, so there's nothing to switch off.
//!  * the terminal size is whatever xterm.js reports; we get it from a host
//!    import (`env.host_terminal_size`, packed as `(cols << 16) | rows`), which
//!    the JS host also feeds the rest of the game.
//!  * progressive keyboard enhancement isn't supported.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::terminal::WindowSize;

static RAW_MODE: AtomicBool = AtomicBool::new(false);

pub(crate) fn is_raw_mode_enabled() -> bool {
    RAW_MODE.load(Ordering::SeqCst)
}

pub(crate) fn enable_raw_mode() -> io::Result<()> {
    RAW_MODE.store(true, Ordering::SeqCst);
    unsafe { host_set_raw_mode(1) };
    Ok(())
}

pub(crate) fn disable_raw_mode() -> io::Result<()> {
    RAW_MODE.store(false, Ordering::SeqCst);
    unsafe { host_set_raw_mode(0) };
    Ok(())
}

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn host_terminal_size() -> i32;
    // Tells the browser host whether to deliver stdin raw or line-edit + echo it
    // (cooked). The host treats "cooked" as the default before any call.
    fn host_set_raw_mode(on: i32);
}

fn host_size() -> (u16, u16) {
    let packed = unsafe { host_terminal_size() };
    let cols = ((packed >> 16) & 0xffff) as u16;
    let rows = (packed & 0xffff) as u16;
    (cols.max(1), rows.max(1))
}

pub(crate) fn size() -> io::Result<(u16, u16)> {
    Ok(host_size())
}

pub(crate) fn window_size() -> io::Result<WindowSize> {
    let (columns, rows) = host_size();
    Ok(WindowSize {
        columns,
        rows,
        width: 0,
        height: 0,
    })
}

#[cfg(feature = "events")]
pub fn supports_keyboard_enhancement() -> io::Result<bool> {
    Ok(false)
}
