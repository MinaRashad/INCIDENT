//! Bridge to the browser host (see `web/worker.js`).
//!
//! When the game is built for `wasm32-wasip1` it can't talk to an OS audio
//! device, spawn processes, or query a real terminal — instead it calls these
//! imports, which the JavaScript host implements on top of Web Audio, browser
//! windows, and xterm.js. The ABI is deliberately tiny and C-flavoured so it's
//! trivial to wire up on the JS side:
//!
//! ```text
//! host_play_sound(category_ptr: i32, category_len: i32, looping: i32) -> i32
//!     Play a random sound from the named category folder (the string is the
//!     value of `SoundCategory::name()`). If `looping` is non-zero, returns a
//!     handle (>0) usable with `host_stop_sound`; otherwise returns 0.
//! host_stop_sound(handle: i32)
//!     Stop a looping sound previously started with `host_play_sound`.
//! host_open_window(name_ptr: i32, name_len: i32)
//!     Open a new game "window" in the given mode ("docs" / "chats"); the host
//!     opens a fresh browser tab booting the page with `?mode=<name>`.
//! host_terminal_size() -> i32
//!     Current terminal size, packed as `(cols << 16) | rows`.
//! ```
//!
//! This module is only compiled for wasm; nothing here affects native builds.

#![cfg(target_arch = "wasm32")]

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn host_play_sound(category_ptr: *const u8, category_len: usize, looping: i32) -> i32;
    fn host_stop_sound(handle: i32);
    fn host_open_window(name_ptr: *const u8, name_len: usize);
    fn host_terminal_size() -> i32;
}

/// Play a random sound from `category` (a `SoundCategory::name()` value).
/// Returns a handle for looping sounds, or `None` for one-shots / failure.
pub fn play_sound(category: &str, looping: bool) -> Option<i32> {
    let h = unsafe {
        host_play_sound(category.as_ptr(), category.len(), if looping { 1 } else { 0 })
    };
    if looping && h > 0 { Some(h) } else { None }
}

pub fn stop_sound(handle: i32) {
    unsafe { host_stop_sound(handle) }
}

/// Ask the host to open a new game window in `mode` (e.g. "docs", "chats").
pub fn open_window(mode: &str) {
    unsafe { host_open_window(mode.as_ptr(), mode.len()) }
}

/// Current terminal size as `[cols, rows]`.
pub fn terminal_size() -> [usize; 2] {
    let packed = unsafe { host_terminal_size() };
    let cols = ((packed >> 16) & 0xffff) as usize;
    let rows = (packed & 0xffff) as usize;
    [cols.max(1), rows.max(1)]
}
