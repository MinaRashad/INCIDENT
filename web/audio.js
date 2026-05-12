// Web Audio bridge — the browser equivalent of the game's rodio-based `sound` module.
//
// The wasm game calls the host import `host_play_sound(categoryName, looping)`,
// where `categoryName` is the folder name produced by `SoundCategory::name()`
// in src/sound.rs ("type", "space", "boot", "gui_feedback", "good", "bad",
// "access_denied", "access_granted", "error", "scroll", "new_message", "sad",
// "low_humming", "loud_humming", "music"). Each category maps to a folder under
// assets/sounds/<category>/ holding one or more audio files; we pick one at
// random per play, exactly like the native game.

export class AudioBridge {
  constructor() {
    this.ctx = null;
    this.buffers = new Map();   // category -> AudioBuffer[]
    this.active = new Map();    // handle id -> { src, gain }
    this.nextId = 1;
    this.suspended = true;
  }

  // `manifest` is { categoryName: [url, url, ...] } produced by build.sh.
  // URLs may contain spaces / apostrophes (asset folder names do) — encode per
  // path segment before fetching.
  async load(manifest) {
    this.ctx = new (window.AudioContext || window.webkitAudioContext)();
    const encUrl = (u) => u.split("/").map((s, i) => (i === 0 && s === ".") ? s : encodeURIComponent(s)).join("/");
    const jobs = [];
    for (const [cat, urls] of Object.entries(manifest)) {
      const list = [];
      this.buffers.set(cat, list);
      for (const url of urls) {
        jobs.push(
          fetch(encUrl(url))
            .then((r) => r.arrayBuffer())
            .then((buf) => this.ctx.decodeAudioData(buf))
            .then((decoded) => list.push(decoded))
            .catch((e) => console.warn("audio: failed to load", url, e))
        );
      }
    }
    await Promise.all(jobs);
  }

  // Call from a user-gesture handler (e.g. first keystroke) — browsers start
  // AudioContexts suspended.
  async resume() {
    if (this.ctx && this.ctx.state === "suspended") {
      try { await this.ctx.resume(); } catch (_) {}
    }
    this.suspended = false;
  }

  // Returns a handle id (>0) for stopping a looping sound, or 0 if nothing played.
  play(category, looping) {
    if (!this.ctx) return 0;
    const list = this.buffers.get(category);
    if (!list || list.length === 0) return 0;
    const buf = list[(Math.random() * list.length) | 0];
    const src = this.ctx.createBufferSource();
    src.buffer = buf;
    src.loop = !!looping;
    const gain = this.ctx.createGain();
    gain.gain.value = 0.7;
    src.connect(gain).connect(this.ctx.destination);
    const id = this.nextId++;
    src.onended = () => {
      if (!src.loop) this.active.delete(id);
    };
    src.start();
    this.active.set(id, { src, gain });
    return looping ? id : 0;
  }

  stop(id) {
    const entry = this.active.get(id);
    if (!entry) return;
    try { entry.src.stop(); } catch (_) {}
    try { entry.src.disconnect(); entry.gain.disconnect(); } catch (_) {}
    this.active.delete(id);
  }

  stopAll() {
    for (const id of [...this.active.keys()]) this.stop(id);
  }
}
