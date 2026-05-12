// Shared protocol between the main thread (host.js) and the game worker (worker.js).
//
// The game is a synchronous WASI program: it calls fd_read on stdin and *blocks*
// until a key is available. Browsers can't block the main thread, so the wasm
// runs in a Web Worker, and stdin is delivered through a SharedArrayBuffer ring
// buffer using Atomics so the worker can block (Atomics.wait) without spinning.
//
// SharedArrayBuffer layout (all little-endian):
//
//   Int32 region (control), indices into the Int32 view:
//     [0] WRITE_IDX  - next write position in the byte ring (main thread owns)
//     [1] READ_IDX   - next read position in the byte ring (worker owns)
//     [2] COLS       - current terminal columns (main thread owns)
//     [3] ROWS       - current terminal rows (main thread owns)
//     [4] FUTEX      - Atomics.wait/notify target; bumped on every stdin write
//                      and on resize so a blocked worker wakes up
//   ... then RING_BYTES of byte storage immediately after the Int32 region.

export const CTRL_INTS = 8;            // a little headroom past the 5 we use
export const RING_BYTES = 1 << 16;     // 64 KiB stdin ring; plenty for keystrokes

export const IDX_WRITE   = 0;
export const IDX_READ    = 1;
export const IDX_COLS    = 2;
export const IDX_ROWS    = 3;
export const IDX_FUTEX   = 4;
export const IDX_RAWMODE = 5;   // 0 = cooked (line-edited, echoed), 1 = raw passthrough

export const SAB_BYTES = CTRL_INTS * 4 + RING_BYTES;
export const RING_OFFSET = CTRL_INTS * 4;

export function makeSab() {
  return new SharedArrayBuffer(SAB_BYTES);
}

// --- main-thread side: push bytes typed by the user into the ring ---------

export function sabPushBytes(sab, bytes) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  const ring = new Uint8Array(sab, RING_OFFSET, RING_BYTES);
  let w = Atomics.load(ctrl, IDX_WRITE);
  const r = Atomics.load(ctrl, IDX_READ);
  // Free space = RING_BYTES - 1 - (w - r) mod RING_BYTES  (one slot kept empty)
  const used = (w - r + RING_BYTES) % RING_BYTES;
  const free = RING_BYTES - 1 - used;
  const n = Math.min(bytes.length, free);
  for (let i = 0; i < n; i++) {
    ring[w] = bytes[i];
    w = (w + 1) % RING_BYTES;
  }
  Atomics.store(ctrl, IDX_WRITE, w);
  Atomics.add(ctrl, IDX_FUTEX, 1);
  Atomics.notify(ctrl, IDX_FUTEX);
  return n;
}

export function sabSetSize(sab, cols, rows) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  Atomics.store(ctrl, IDX_COLS, cols | 0);
  Atomics.store(ctrl, IDX_ROWS, rows | 0);
  Atomics.add(ctrl, IDX_FUTEX, 1);
  Atomics.notify(ctrl, IDX_FUTEX);
}

// --- worker side: blocking read of up to `max` bytes ----------------------

export function sabReadBlocking(sab, max) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  const ring = new Uint8Array(sab, RING_OFFSET, RING_BYTES);
  for (;;) {
    const w = Atomics.load(ctrl, IDX_WRITE);
    let r = Atomics.load(ctrl, IDX_READ);
    if (w !== r) {
      const out = [];
      while (r !== w && out.length < max) {
        out.push(ring[r]);
        r = (r + 1) % RING_BYTES;
      }
      Atomics.store(ctrl, IDX_READ, r);
      return Uint8Array.from(out);
    }
    // nothing available: park until the main thread bumps FUTEX
    const seen = Atomics.load(ctrl, IDX_FUTEX);
    Atomics.wait(ctrl, IDX_FUTEX, seen, 1000);
  }
}

export function sabGetSize(sab) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  return [Atomics.load(ctrl, IDX_COLS) || 80, Atomics.load(ctrl, IDX_ROWS) || 24];
}

// Number of bytes currently sitting unread in the stdin ring.
export function sabBytesAvailable(sab) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  const w = Atomics.load(ctrl, IDX_WRITE);
  const r = Atomics.load(ctrl, IDX_READ);
  return (w - r + RING_BYTES) % RING_BYTES;
}

// Block the calling (worker) thread for up to `ms` milliseconds, or until the
// FUTEX is bumped (i.e. new stdin / a resize). Returns "timed-out" | "ok" |
// "not-equal". `ms === Infinity` blocks indefinitely.
export function sabPark(sab, ms) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  const seen = Atomics.load(ctrl, IDX_FUTEX);
  return Atomics.wait(ctrl, IDX_FUTEX, seen, ms);
}

// A standalone synchronous sleep (no FUTEX involved) — used to implement WASI
// clock waits in poll_oneoff. Worker-thread only (Atomics.wait isn't allowed on
// the main thread).
const _sleeper = new Int32Array(new SharedArrayBuffer(4));
export function sabSleep(ms) {
  if (ms <= 0) return;
  Atomics.wait(_sleeper, 0, 0, ms);
}

// --- terminal "line discipline" mode -------------------------------------
// The wasm's crossterm enable_raw_mode/disable_raw_mode flip this (via a host
// import). The main thread reads it to decide whether the user's keystrokes go
// straight through (raw: menus, ratatui views) or get line-edited + echoed
// before being delivered as a whole line (cooked: `read_line` password prompt,
// `wait_for_input`). Defaults to 0 = cooked, matching a fresh tty.

export function sabSetRawMode(sab, on) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  Atomics.store(ctrl, IDX_RAWMODE, on ? 1 : 0);
}

export function sabIsRawMode(sab) {
  const ctrl = new Int32Array(sab, 0, CTRL_INTS);
  return Atomics.load(ctrl, IDX_RAWMODE) === 1;
}
