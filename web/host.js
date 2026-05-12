// host.js — main-thread browser host for the INCIDENT terminal game.
//
// This is the "browser view" component: it owns the on-screen presentation
// (an xterm.js terminal styled to match the game's aesthetic) and the I/O
// plumbing, but it does not touch game logic. The unmodified game runs as
// wasm inside worker.js; this file just renders its stdout, feeds it the
// keystrokes, plays its sounds via Web Audio, and persists its save file.

import {
  makeSab, sabPushBytes, sabSetSize, sabIsRawMode, RING_OFFSET,
} from "./bridge.js";
import { AudioBridge } from "./audio.js";
import { loadSave, writeSave, clearSave } from "./persist.js";

const TERM_THEME = {
  background: "#000000",
  foreground: "#c8c8c8",
  cursor: "#64fa64",
  cursorAccent: "#000000",
  selectionBackground: "#264f78",
  black: "#000000",
  red: "#ff3232",
  green: "#32ff32",
  yellow: "#ffc800",
  blue: "#3c78ff",
  magenta: "#c850c8",
  cyan: "#00c8ff",
  white: "#c8c8c8",
  brightBlack: "#5a5a5a",
  brightGreen: "#64fa64",
  brightWhite: "#ffffff",
};

const params = new URLSearchParams(location.search);
const MODE = params.get("mode"); // "docs" | "chats" | null

const bootEl = document.getElementById("boot");

const term = new Terminal({
  theme: TERM_THEME,
  fontFamily: '"Pixelify Sans", "Cascadia Mono", Consolas, monospace',
  fontSize: 16,
  cursorBlink: true,
  // The game prints bare "\n" (Rust's println!) and counts on the OS terminal
  // driver's ONLCR to add the "\r". xterm.js doesn't translate, so a bare "\n"
  // would move down without returning to column 0 — staircased text. convertEol
  // makes "\n" behave like "\r\n" (and an explicit "\r\n" still works fine:
  // "\r" -> col 0, then "\n" -> col 0 + down).
  convertEol: true,
  scrollback: 5000,
  allowProposedApi: true,
});
const fit = new FitAddon.FitAddon();
term.loadAddon(fit);
term.loadAddon(new WebLinksAddon.WebLinksAddon());
term.open(document.getElementById("terminal"));
fit.fit();

const sab = makeSab();
sabSetSize(sab, term.cols, term.rows);

const audio = new AudioBridge();

// ---- worker wiring ---------------------------------------------------------

const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
let started = false;

const soundHandles = new Map();   // wasm sound id -> audio bridge handle id
let saveSlot = {};                // accumulated { "main.db": Uint8Array, ... }
let saveTimer = null;
function scheduleSave() {
  if (saveTimer) return;
  saveTimer = setTimeout(() => {
    saveTimer = null;
    console.log("[host] Writing to IndexedDB...", Object.keys(saveSlot));
    writeSave(saveSlot)
      .then(() => console.log("[host] IndexedDB write successful"))
      .catch((e) => console.error("[host] IndexedDB write FAILED", e));
  }, 100);
}

worker.onmessage = (ev) => {
  const m = ev.data;
  switch (m.type) {
    case "stdout":
      term.write(m.bytes); // Uint8Array; xterm decodes UTF-8
      break;
    case "play_sound": {
      const audioId = audio.play(m.category, m.looping);
      if (m.looping) soundHandles.set(m.wasmId, audioId);
      break;
    }
    case "stop_sound": {
      const audioId = soundHandles.get(m.wasmId);
      if (audioId) audio.stop(audioId);
      soundHandles.delete(m.wasmId);
      break;
    }
    case "open_window":
      // Desktop spawns a fresh OS terminal running `INCIDENT --<mode>`; in the
      // browser we open a new tab booting the same page in that mode.
      window.open(`${location.pathname}?mode=${encodeURIComponent(m.mode)}`, "_blank");
      break;
    case "save_file":
      console.log(`[host] Received save_file: ${m.name} (${m.data.byteLength} bytes)`);
      saveSlot[m.name] = m.data;
      scheduleSave();
      break;
    case "save":
      console.log("[host] Received final save snapshot", Object.keys(m.files));
      saveSlot = { ...saveSlot, ...m.files };
      writeSave(saveSlot)
        .then(() => console.log("[host] Final IndexedDB write successful"))
        .catch((e) => console.error("[host] Final IndexedDB write FAILED", e));
      break;
    case "exit":
      term.write(`\r\n\x1b[90m[ INCIDENT exited (${m.code}) ]\x1b[0m\r\n`);
      audio.stopAll();
      break;
    case "ready":
      bootEl.classList.add("hidden");
      break;
    case "error":
      bootEl.classList.remove("hidden");
      bootEl.textContent = "ERROR: " + m.message;
      console.error("worker error:", m.message, m.stack);
      break;
  }
};

(async () => {
  let manifest = { wasm: "./dist/incident.wasm", assets: "./dist/assets-manifest.json", sounds: {} };
  try {
    const resp = await fetch("./dist/manifest.json");
    if (resp.ok) manifest = await resp.json();
  } catch (_) { /* dev fallback below */ }

  // Preload sounds (best-effort; missing files just play nothing).
  if (manifest.sounds) {
    try { await audio.load(manifest.sounds); } catch (e) { console.warn("audio preload:", e); }
  }

  const save = await loadSave().catch(() => ({}));
  saveSlot = { ...save };

  worker.postMessage({
    type: "init",
    sab,
    mode: MODE,
    wasmUrl: manifest.wasm,
    assetsManifestUrl: manifest.assets,
    save,
  });
})();

// ---- input -----------------------------------------------------------------
//
// The wasm game runs the terminal in two modes (it flips crossterm's raw mode):
//   * raw   — menus, ratatui views: keystrokes go straight to stdin, no echo.
//   * cooked — `read_line` (the password prompt) and `wait_for_input`: the OS
//     terminal driver would line-edit + echo and deliver a whole line ending in
//     "\n". There's no OS driver here, so we emulate that line discipline.
// `host_set_raw_mode` (via the worker) keeps `sabIsRawMode` in sync with the
// wasm's raw-mode state.

const enc = new TextEncoder();
let lineBuf = "";

function ensureStarted() {
  if (started) return;
  started = true;
  audio.resume(); // needs a user gesture; the first keystroke is one
}

function feedInput(data) {
  ensureStarted();

  if (sabIsRawMode(sab)) {
    lineBuf = "";
    sabPushBytes(sab, enc.encode(data));
    return;
  }

  // Cooked: ignore multi-byte escape sequences (arrows, function keys, …).
  if (data.length > 1 && data.charCodeAt(0) === 0x1b) return;

  for (const ch of data) {
    const c = ch.charCodeAt(0);
    if (c === 0x0d || c === 0x0a) {
      // Enter: echo a newline, deliver the line (with "\n", which crossterm
      // also reads as Enter while not in raw mode), reset.
      term.write("\r\n");
      sabPushBytes(sab, enc.encode(lineBuf + "\n"));
      lineBuf = "";
    } else if (c === 0x7f || c === 0x08) {
      // Backspace / Delete.
      if (lineBuf.length > 0) {
        lineBuf = lineBuf.slice(0, -1);
        term.write("\b \b");
      }
    } else if (c === 0x15) {
      // Ctrl-U: kill the whole line.
      while (lineBuf.length > 0) { lineBuf = lineBuf.slice(0, -1); term.write("\b \b"); }
    } else if (c === 0x03) {
      // Ctrl-C: pass it through so the program can see it.
      sabPushBytes(sab, enc.encode("\x03"));
    } else if (c >= 0x20) {
      lineBuf += ch;
      term.write(ch);
    }
    // other control chars: ignored in cooked mode
  }
}

term.onData(feedInput);

// Drop auto-repeat keydowns. The game acts on key *releases* (it's Windows-first
// and our wasi backend synthesises a release after each press), so a held key
// would otherwise emit a stream of press/release pairs and skip through screens.
// Returning false here cancels xterm's default handling, so onData doesn't fire
// for the repeat.
term.attachCustomKeyEventHandler((e) => !(e.type === "keydown" && e.repeat));

// ---- resize ----------------------------------------------------------------

const resize = () => {
  fit.fit();
  sabSetSize(sab, term.cols, term.rows);
};
window.addEventListener("resize", resize);
term.onResize(({ cols, rows }) => sabSetSize(sab, cols, rows));

// ---- dev convenience -------------------------------------------------------

window.INCIDENT = {
  term,
  resetSave: async () => { await clearSave(); location.reload(); },
};
