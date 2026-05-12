// worker.js — runs the INCIDENT game (compiled to wasm32-wasip1) inside a Web
// Worker, behind a WASI shim, so its synchronous/blocking I/O model works in a
// browser without freezing the page.
//
// Responsibilities:
//   * build an in-memory WASI filesystem from the bundled assets + the saved DB
//   * stdin  : blocking reads served from a SharedArrayBuffer ring (keystrokes)
//   * stdout : forwarded to the main thread as raw bytes (xterm.js renders them)
//   * host imports (`env` module): sound, "open new window", terminal size
//   * poll_oneoff: not provided by browser_wasi_shim — we supply one so that
//     `std::thread::sleep` (timed animations, pacing) and `event::poll(timeout)`
//     work; clock waits sleep the worker via Atomics.wait, fd waits block on the
//     stdin ring.
//   * write-through persistence of main.db back to the main thread
//
// The game's own Rust code is unchanged; only `windows.rs` (process-spawning),
// `sound.rs` (rodio) and `terminal::size()` are cfg-gated for wasm, plus a small
// `wasm_host.rs` declares the host imports; the vendored crossterm gets a `wasi`
// `sys` backend (raw-mode no-op + a stdin-byte event source).

import {
  WASI, Fd, File, Directory, PreopenDirectory, wasi,
} from "https://esm.sh/@bjorn3/browser_wasi_shim@0.3.0";

import { sabReadBlocking, sabGetSize, sabBytesAvailable, sabPark, sabSleep, sabSetRawMode } from "./bridge.js";

let SAB = null;
const PERSIST_FILES = ["main.db", "main.db-wal", "main.db-shm"];

// --- custom file descriptors ------------------------------------------------

function charDeviceFdstat() {
  // FILETYPE_CHARACTER_DEVICE = 2
  return new wasi.Fdstat(2 /* FILETYPE_CHARACTER_DEVICE */, 0);
}

class StdinFd extends Fd {
  fd_fdstat_get() { return { ret: 0, fdstat: charDeviceFdstat() }; }
  fd_read(size) {
    if (!SAB) return { ret: 0, data: new Uint8Array(0) };
    const data = sabReadBlocking(SAB, size);
    return { ret: 0, data };
  }
}

class StdoutFd extends Fd {
  constructor(kind) { super(); this.kind = kind; } // "stdout" | "stderr"
  fd_fdstat_get() { return { ret: 0, fdstat: charDeviceFdstat() }; }
  fd_write(data) {
    // `data` may be a view into wasm memory — copy before transferring.
    const bytes = data.slice();
    if (this.kind === "stderr") {
      // surface panics etc. to the JS console; don't paint them on the terminal
      try { console.log("[game]", new TextDecoder().decode(bytes)); } catch (_) {}
    } else {
      postMessage({ type: "stdout", bytes }, [bytes.buffer]);
    }
    return { ret: 0, nwritten: data.byteLength };
  }
}

// A File whose contents are mirrored back to the main thread on every write,
// so progress is saved even though timers can't fire while _start() runs.
class PersistedFile extends File {
  constructor(name, data) { super(data ?? new Uint8Array(0)); this.persistName = name; }
  fd_write(...args) { const r = super.fd_write(...args); this._flush(); return r; }
  fd_pwrite(...args) { const r = super.fd_pwrite(...args); this._flush(); return r; }
  fd_allocate(...args) { const r = super.fd_allocate(...args); this._flush(); return r; }
  fd_filestat_set_size(...args) { const r = super.fd_filestat_set_size(...args); this._flush(); return r; }
  fd_datasync() { this._flush(); return { ret: 0 }; }
  fd_sync() { this._flush(); return { ret: 0 }; }
  _flush() {
    try { 
      console.log(`[worker] Flushing ${this.persistName}, size: ${this.data.byteLength} bytes`);
      postMessage({ type: "save_file", name: this.persistName, data: this.data.slice() }); 
    }
    catch (e) { console.error(`[worker] Flush failed for ${this.persistName}:`, e); }
  }
}

// --- filesystem assembly ----------------------------------------------------

// Insert `path` (e.g. "assets/documents/foo.txt") -> File into the directory
// tree rooted at the given Map, creating intermediate Directory nodes.
function insertPath(rootMap, path, file) {
  const parts = path.split("/").filter(Boolean);
  let map = rootMap;
  for (let i = 0; i < parts.length - 1; i++) {
    let node = map.get(parts[i]);
    if (!(node instanceof Directory)) { node = new Directory(new Map()); map.set(parts[i], node); }
    map = node.contents;
  }
  map.set(parts[parts.length - 1], file);
}

async function buildFs({ assetsManifestUrl, save }) {
  const root = new Map();

  // default.sql (schema) — ships with the build under dist/
  try {
    const r = await fetch("./dist/default.sql");
    if (r.ok) root.set("default.sql", new File(new Uint8Array(await r.arrayBuffer())));
  } catch (_) {}

  // assets/* (documents, chat dialogue, OS logo image, fonts, …) — sounds are
  // handled by the Web Audio bridge on the main thread, so they're excluded.
  const enc = (rel) => rel.split("/").map(encodeURIComponent).join("/");
  try {
    const r = await fetch(assetsManifestUrl);
    if (r.ok) {
      const list = await r.json(); // ["assets/documents/...", "assets/Chat/dialogue.json", ...]
      await Promise.all(list.map(async (rel) => {
        try {
          const fr = await fetch("./dist/" + enc(rel));
          if (fr.ok) insertPath(root, rel, new File(new Uint8Array(await fr.arrayBuffer())));
        } catch (_) {}
      }));
    }
  } catch (_) {}

  // restore the saved SQLite database (if any). If there's no save we still
  // create an empty PersistedFile for main.db so that the game's initialization
  // is mirrored to IndexedDB from the first write.
  for (const name of PERSIST_FILES) {
    if (name === "main.db" || (save && save[name])) {
      const data = (save && save[name]) ? new Uint8Array(save[name]) : new Uint8Array(0);
      root.set(name, new PersistedFile(name, data));
    }
  }

  return root;
}

// --- main -------------------------------------------------------------------

let memory = null;
let nextSoundId = 1;
const cstr = (ptr, len) => new TextDecoder().decode(new Uint8Array(memory.buffer, ptr >>> 0, len >>> 0));

function makeEnvImports() {
  return {
    // see src/wasm_host.rs
    host_play_sound(catPtr, catLen, looping) {
      const category = cstr(catPtr, catLen);
      const wasmId = nextSoundId++;
      postMessage({ type: "play_sound", category, looping: looping !== 0, wasmId });
      return looping !== 0 ? wasmId : 0; // 0 == "nothing to stop"
    },
    host_stop_sound(wasmId) {
      if (wasmId) postMessage({ type: "stop_sound", wasmId });
    },
    host_open_window(namePtr, nameLen) {
      postMessage({ type: "open_window", mode: cstr(namePtr, nameLen) });
    },
    host_terminal_size() {
      const [c, r] = SAB ? sabGetSize(SAB) : [80, 24];
      return (((c & 0xffff) << 16) | (r & 0xffff)) | 0;
    },
    // crossterm's wasi backend calls this from enable_raw_mode/disable_raw_mode
    // so the main thread knows whether to line-edit + echo stdin.
    host_set_raw_mode(on) {
      if (SAB) sabSetRawMode(SAB, on);
    },
  };
}

// A minimal `poll_oneoff` covering the cases this game actually hits:
//  * clock subscription(s) only  -> sleep the worker for the shortest timeout
//    (Atomics.wait), then report every clock subscription as fired. This is
//    what `std::thread::sleep` compiles to.
//  * fd_read subscription(s)     -> if stdin already has bytes report it ready
//    immediately; otherwise park on the stdin ring (up to the shortest clock
//    timeout if one is also present), then report whichever fired.
//  * fd_write subscription(s)    -> always reported ready (we only ever write
//    stdout / in-memory files).
// Struct layout per the WASI preview1 ABI: subscription = 48 bytes
// (userdata u64 @0, tag u8 @8, then clock{id u32 @16, timeout u64 @24,
// precision u64 @32, flags u16 @40} or fd{fd u32 @16}); event = 32 bytes
// (userdata u64 @0, errno u16 @8, type u8 @10, fd_readwrite{nbytes u64 @16,
// flags u16 @24}).
function makePollOneoff(getMemory, sab) {
  const EVENTTYPE_CLOCK = 0, EVENTTYPE_FD_READ = 1, EVENTTYPE_FD_WRITE = 2;
  const SUBCLOCKFLAGS_ABSTIME = 1;
  return function poll_oneoff(inPtr, outPtr, nsub, neventsPtr) {
    const dv = new DataView(getMemory().buffer);
    let minTimeoutMs = Infinity;
    const clockSubs = [];
    const fdSubs = [];
    for (let i = 0; i < nsub; i++) {
      const base = inPtr + i * 48;
      const userdata = dv.getBigUint64(base + 0, true);
      const tag = dv.getUint8(base + 8);
      if (tag === EVENTTYPE_CLOCK) {
        const timeoutNs = dv.getBigUint64(base + 24, true);
        const flags = dv.getUint16(base + 40, true);
        let ms = Number(timeoutNs / 1000000n);
        // We can't reliably know a clock's epoch here, so treat ABSTIME the
        // same as a (small) relative wait — the game only uses relative sleeps.
        if (flags & SUBCLOCKFLAGS_ABSTIME) ms = Math.max(0, ms);
        if (ms < minTimeoutMs) minTimeoutMs = ms;
        clockSubs.push(userdata);
      } else { // fd_read or fd_write
        const fd = dv.getUint32(base + 16, true);
        fdSubs.push({ userdata, fd, write: tag === EVENTTYPE_FD_WRITE });
      }
    }

    const stdinHas = () => sabBytesAvailable(sab) > 0;

    if (fdSubs.length > 0) {
      // Only actually block if every fd subscription is a stdin read and stdin
      // is currently empty; otherwise some fd is "ready" (writes, or other fds)
      // so we return immediately.
      const onlyStdinReads = fdSubs.every((s) => s.fd === 0 && !s.write);
      if (onlyStdinReads && !stdinHas()) {
        sabPark(sab, minTimeoutMs === Infinity ? Infinity : Math.max(0, minTimeoutMs));
      }
    } else if (clockSubs.length > 0) {
      sabSleep(minTimeoutMs === Infinity ? 0 : Math.max(0, minTimeoutMs));
    }

    const events = [];
    for (const s of fdSubs) {
      if (s.write) {
        events.push({ userdata: s.userdata, type: EVENTTYPE_FD_WRITE, nbytes: 0n });
      } else if (s.fd === 0) {
        const avail = sabBytesAvailable(sab);
        if (avail > 0) events.push({ userdata: s.userdata, type: EVENTTYPE_FD_READ, nbytes: BigInt(avail) });
      } else {
        // unknown read fd: report ready with 0 bytes so the caller proceeds
        events.push({ userdata: s.userdata, type: EVENTTYPE_FD_READ, nbytes: 0n });
      }
    }
    if (events.length === 0) {
      // either it was a pure sleep, or stdin had nothing when the clock elapsed
      for (const u of clockSubs) events.push({ userdata: u, type: EVENTTYPE_CLOCK, nbytes: 0n });
    }

    let n = 0;
    for (const ev of events) {
      const o = outPtr + n * 32;
      dv.setBigUint64(o + 0, ev.userdata, true);
      dv.setUint16(o + 8, 0, true);            // errno = success
      dv.setUint8(o + 10, ev.type);
      dv.setUint8(o + 11, 0);                  // pad
      dv.setBigUint64(o + 16, ev.nbytes ?? 0n, true);
      dv.setUint16(o + 24, 0, true);           // fd_readwrite.flags
      n++;
    }
    dv.setUint32(neventsPtr, n, true);
    return 0; // ERRNO_SUCCESS
  };
}

async function run({ sab, mode, wasmUrl, assetsManifestUrl, save }) {
  SAB = sab;

  const rootContents = await buildFs({ assetsManifestUrl, save });
  // Preopen the same tree as both "/" and "." so the game's relative paths
  // ("default.sql", "main.db", "assets/...") resolve regardless of how
  // wasi-libc derives the cwd.
  const fds = [
    new StdinFd(),
    new StdoutFd("stdout"),
    new StdoutFd("stderr"),
    new PreopenDirectory("/", rootContents),
    new PreopenDirectory(".", rootContents),
  ];

  const [cols, rows] = sabGetSize(sab);
  const args = ["incident", ...(mode ? [`--${mode}`] : [])];
  const env = [`INCIDENT_COLS=${cols}`, `INCIDENT_LINES=${rows}`, "TERM=xterm-256color", "RUST_BACKTRACE=1"];

  const w = new WASI(args, env, fds);
  // browser_wasi_shim throws on poll_oneoff; supply a real one so thread::sleep
  // and timed event polling work.
  w.wasiImport.poll_oneoff = makePollOneoff(() => memory, sab);

  // --- persistence interception -------------------------------------------
  // The WASI shim's File class doesn't always call our overridden methods.
  // We wrap the raw WASI imports to ensure every write to a persisted file
  // triggers a flush to the main thread.
  const wrapFlush = (original) => {
    return (...args) => {
      const fd = args[0];
      const ret = original(...args);
      if (ret === 0) {
        // Find the file associated with this FD
        const file = w.fds[fd]?.file;
        if (file && file.persistName) {
          file._flush();
        }
      }
      return ret;
    };
  };
  w.wasiImport.fd_write = wrapFlush(w.wasiImport.fd_write);
  w.wasiImport.fd_pwrite = wrapFlush(w.wasiImport.fd_pwrite);
  w.wasiImport.fd_datasync = wrapFlush(w.wasiImport.fd_datasync);
  w.wasiImport.fd_sync = wrapFlush(w.wasiImport.fd_sync);

  const importObject = {
    wasi_snapshot_preview1: w.wasiImport,
    env: makeEnvImports(),
  };

  const { instance } = await WebAssembly.instantiateStreaming(fetch(wasmUrl), importObject)
    .catch(async () => {
      // fallback for servers without application/wasm mime type
      const bytes = await (await fetch(wasmUrl)).arrayBuffer();
      return WebAssembly.instantiate(bytes, importObject);
    });

  memory = instance.exports.memory;
  postMessage({ type: "ready" });

  let code = 0;
  try {
    code = w.start(instance);
  } catch (e) {
    if (e && e.name === "WASIProcExit") code = e.code ?? 0;
    else { postMessage({ type: "error", message: String(e && e.message || e), stack: e && e.stack }); return; }
  }

  // final snapshot of the DB
  const dir = rootContents;
  const files = {};
  for (const name of PERSIST_FILES) {
    const f = dir.get(name);
    if (f instanceof File) files[name] = f.data.slice();
  }
  postMessage({ type: "save", files });
  postMessage({ type: "exit", code });
}

onmessage = (ev) => {
  const m = ev.data;
  if (m.type === "init") {
    run(m).catch((e) => postMessage({ type: "error", message: String(e && e.message || e), stack: e && e.stack }));
  }
};
