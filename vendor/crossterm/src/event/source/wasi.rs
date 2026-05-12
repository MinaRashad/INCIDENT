//! wasm/wasi event source.
//!
//! In the browser there's no tty, no termios and no `mio`. The JS host
//! (xterm.js) writes the user's keystrokes to the wasm module's stdin as raw
//! byte sequences — the same bytes a real terminal would send — so this source
//! just reads stdin and runs them through crossterm's own ANSI parser
//! (`parse_event`, shared with the unix backend).
//!
//! Key-release synthesis: a raw terminal only ever reports key *presses*
//! (`KeyEventKind::Press`), whereas the Windows console reports a press *and*
//! a release for every keystroke. Lots of TUI code (including INCIDENT, which
//! is Windows-first) only acts on the release — so for each parsed key press we
//! also queue a matching `KeyEventKind::Release`, making this source behave
//! like the Windows backend.
//!
//! Timed polling: `event::poll(Some(d))` needs `try_read` to return after at
//! most `d`. We can't time out a bare `fd_read`, so we first call WASI
//! `poll_oneoff` with a clock subscription (`d`) plus an `fd_read` subscription
//! on stdin; only if `poll_oneoff` says stdin is readable do we actually read
//! (which then won't block). The browser host (`web/worker.js`) implements
//! `poll_oneoff` on top of a SharedArrayBuffer so the wasm worker can block
//! without freezing the page. `poll(Duration::ZERO)` stays a cheap non-blocking
//! probe of the already-parsed queue.

use std::collections::VecDeque;
use std::io::{self, Read};
use std::time::Duration;

#[cfg(feature = "event-stream")]
use crate::event::sys::Waker;
use crate::event::sys::wasi::parse::parse_event;
use crate::event::{source::EventSource, Event, InternalEvent, KeyEventKind};

const STDIN_BUFFER_SIZE: usize = 1024;

pub(crate) struct WasiInternalEventSource {
    parser: Parser,
    read_buf: [u8; STDIN_BUFFER_SIZE],
}

impl WasiInternalEventSource {
    pub(crate) fn new() -> io::Result<Self> {
        Ok(WasiInternalEventSource {
            parser: Parser::default(),
            read_buf: [0u8; STDIN_BUFFER_SIZE],
        })
    }
}

impl EventSource for WasiInternalEventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<InternalEvent>> {
        if let Some(event) = self.parser.next() {
            return Ok(Some(event));
        }

        // Wait (up to `timeout`, or indefinitely if `None`) for stdin to be
        // readable. A `Some(ZERO)` timeout makes this a non-blocking probe
        // (used by `poll(0)` / `drain_input` / EventStream) — but it still
        // consumes and parses any bytes already sitting in stdin, which is what
        // makes `drain_input()` actually drain stale keystrokes.
        if !wait_for_stdin(timeout) {
            return Ok(None); // nothing available within the deadline
        }

        // stdin is readable now; this read won't block.
        let mut stdin = io::stdin().lock();
        let n = match stdin.read(&mut self.read_buf) {
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => return Ok(None),
            Err(e) => return Err(e),
        };
        if n == 0 {
            // EOF on stdin shouldn't happen in the browser; report "no event".
            return Ok(None);
        }
        self.parser
            .advance(&self.read_buf[..n], n == STDIN_BUFFER_SIZE);
        Ok(self.parser.next())
    }

    #[cfg(feature = "event-stream")]
    fn waker(&self) -> Waker {
        unimplemented!("event-stream is not supported on wasm")
    }
}

// --- WASI poll_oneoff (hand-rolled; the struct layouts are small and stable) -

#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    fn poll_oneoff(
        in_: *const u8,
        out: *mut u8,
        nsubscriptions: usize,
        nevents: *mut usize,
    ) -> i32;
}

// subscription_t = 48 bytes:
//   userdata u64 @0
//   u.tag    u8  @8   (0=clock, 1=fd_read, 2=fd_write)
//   union    @16:
//     clock: id u32 @16, timeout u64 @24, precision u64 @32, flags u16 @40
//     fd:    file_descriptor u32 @16
// event_t = 32 bytes:
//   userdata u64 @0, error u16 @8, type u8 @10, fd_readwrite{nbytes u64 @16, flags u16 @24}
const SUB_SIZE: usize = 48;
const EVT_SIZE: usize = 32;
const EVENTTYPE_CLOCK: u8 = 0;
const EVENTTYPE_FD_READ: u8 = 1;
const CLOCKID_MONOTONIC: u32 = 1;
const UD_STDIN: u64 = 1;
const UD_CLOCK: u64 = 2;

/// Block (up to `timeout`, or indefinitely if `None`) until stdin (fd 0) is
/// readable. Returns `true` if it became readable, `false` on timeout.
fn wait_for_stdin(timeout: Option<Duration>) -> bool {
    let mut subs = [0u8; SUB_SIZE * 2];

    // [0] fd_read on stdin
    subs[0..8].copy_from_slice(&UD_STDIN.to_le_bytes());
    subs[8] = EVENTTYPE_FD_READ;
    subs[16..20].copy_from_slice(&0u32.to_le_bytes()); // fd 0

    let nsub = if let Some(d) = timeout {
        // [1] monotonic clock, relative timeout
        let s = &mut subs[SUB_SIZE..SUB_SIZE * 2];
        s[0..8].copy_from_slice(&UD_CLOCK.to_le_bytes());
        s[8] = EVENTTYPE_CLOCK;
        s[16..20].copy_from_slice(&CLOCKID_MONOTONIC.to_le_bytes());
        let nanos = u64::try_from(d.as_nanos()).unwrap_or(u64::MAX);
        s[24..32].copy_from_slice(&nanos.to_le_bytes());
        // precision @32 = 0, flags @40 = 0 (relative)
        2
    } else {
        1
    };

    let mut out = [0u8; EVT_SIZE * 2];
    let mut nevents: usize = 0;
    let rc = unsafe { poll_oneoff(subs.as_ptr(), out.as_mut_ptr(), nsub, &mut nevents) };
    if rc != 0 {
        // poll_oneoff failed — fall back to "assume readable" (the caller's
        // read may then block, which is the pre-poll_oneoff behaviour).
        return true;
    }
    for i in 0..nevents {
        let e = &out[i * EVT_SIZE..(i + 1) * EVT_SIZE];
        let ud = u64::from_le_bytes(e[0..8].try_into().unwrap());
        let ty = e[10];
        if ty == EVENTTYPE_FD_READ && ud == UD_STDIN {
            return true;
        }
    }
    false // only the clock fired (or nothing) => timeout
}

// --- parser (mirrors the unix mio backend's small `Parser` helper) ----------

#[derive(Debug)]
struct Parser {
    buffer: Vec<u8>,
    internal_events: VecDeque<InternalEvent>,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            buffer: Vec::with_capacity(256),
            internal_events: VecDeque::with_capacity(32),
        }
    }
}

impl Parser {
    fn advance(&mut self, buffer: &[u8], more: bool) {
        for (idx, byte) in buffer.iter().enumerate() {
            let more = idx + 1 < buffer.len() || more;
            self.buffer.push(*byte);
            match parse_event(&self.buffer, more) {
                Ok(Some(ie)) => {
                    self.push(ie);
                    self.buffer.clear();
                }
                Ok(None) => { /* need more bytes */ }
                Err(_) => {
                    self.buffer.clear();
                }
            }
        }
    }

    fn push(&mut self, ie: InternalEvent) {
        // Mimic the Windows backend: every key press is followed by a release.
        if let InternalEvent::Event(Event::Key(ke)) = ie {
            if ke.kind == KeyEventKind::Press {
                self.internal_events
                    .push_back(InternalEvent::Event(Event::Key(ke)));
                self.internal_events.push_back(InternalEvent::Event(Event::Key(
                    crate::event::KeyEvent { kind: KeyEventKind::Release, ..ke },
                )));
                return;
            }
        }
        self.internal_events.push_back(ie);
    }
}

impl Iterator for Parser {
    type Item = InternalEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal_events.pop_front()
    }
}
