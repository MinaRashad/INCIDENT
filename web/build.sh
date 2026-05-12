#!/usr/bin/env bash
# Build the browser bundle for INCIDENT.
#
#   1. compiles the game to wasm32-wasip1
#   2. copies the wasm, the SQL schema and the runtime assets into web/dist/
#   3. generates the manifests host.js / worker.js consume
#
# Run from anywhere; paths are resolved relative to this script.
#
# Prereqs:
#   * rustup target add wasm32-wasip1
#   * a WASI sysroot + clang to compile the bundled SQLite C source. Get a
#     wasi-sdk release (https://github.com/WebAssembly/wasi-sdk/releases) and
#     point WASI_SDK_PATH at it, e.g.:
#         export WASI_SDK_PATH=$HOME/wasi-sdk
#     (Everything Rust — crossterm via vendor/crossterm, ratatui, the game —
#     builds with the stock toolchain; only the C SQLite needs wasi-sdk.)
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
crate_root="$(cd "$here/.." && pwd)"
dist="$here/dist"
profile="${1:-release}"

# --- locate wasi-sdk for the bundled-SQLite C compile ----------------------
: "${WASI_SDK_PATH:=$HOME/wasi-sdk}"
if [[ ! -x "$WASI_SDK_PATH/bin/clang" ]]; then
  echo "!! WASI_SDK_PATH ($WASI_SDK_PATH) doesn't contain bin/clang." >&2
  echo "   Download a wasi-sdk release and set WASI_SDK_PATH, e.g.:" >&2
  echo "     curl -L https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-25/wasi-sdk-25.0-x86_64-linux.tar.gz | tar xz" >&2
  echo "     export WASI_SDK_PATH=\$PWD/wasi-sdk-25.0-x86_64-linux" >&2
  exit 1
fi
export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
export AR_wasm32_wasip1="$WASI_SDK_PATH/bin/llvm-ar"
export CARGO_TARGET_WASM32_WASIP1_LINKER="$WASI_SDK_PATH/bin/clang"

echo ">> building $crate_root for wasm32-wasip1 ($profile)  [wasi-sdk: $WASI_SDK_PATH]"
( cd "$crate_root" && cargo build --target wasm32-wasip1 ${profile:+--$profile} )

wasm_in="$crate_root/target/wasm32-wasip1/$profile/INCIDENT.wasm"
if [[ ! -f "$wasm_in" ]]; then
  echo "!! $wasm_in not found — did the wasm build succeed?" >&2
  exit 1
fi

rm -rf "$dist"
mkdir -p "$dist"
cp "$wasm_in" "$dist/incident.wasm"
# The web version needs WAL disabled to ensure all data is written to the main 
# main.db file for reliable IndexedDB persistence.
grep -v "PRAGMA journal_mode=WAL;" "$crate_root/default.sql" > "$dist/default.sql"
cp -r "$crate_root/assets" "$dist/assets"

echo ">> generating manifests"
python3 - "$dist" <<'PY'
import json, os, sys
dist = sys.argv[1]
assets_root = os.path.join(dist, "assets")

# WASI-FS files: everything under assets/ EXCEPT assets/sounds/** (those are
# played via Web Audio on the main thread, not read by the wasm game).
fs_files = []
sounds = {}
for root, _dirs, files in os.walk(assets_root):
    for f in files:
        full = os.path.join(root, f)
        rel = os.path.relpath(full, dist).replace(os.sep, "/")   # e.g. assets/documents/foo
        parts = rel.split("/")
        if len(parts) >= 3 and parts[0] == "assets" and parts[1] == "sounds":
            # only files inside a category subfolder count (mirrors src/sound.rs)
            if len(parts) >= 4:
                sounds.setdefault(parts[2], []).append("./dist/" + rel)
        else:
            fs_files.append(rel)

fs_files.sort()
for k in sounds: sounds[k].sort()

with open(os.path.join(dist, "assets-manifest.json"), "w") as fh:
    json.dump(fs_files, fh, indent=0)

with open(os.path.join(dist, "manifest.json"), "w") as fh:
    json.dump({
        "wasm": "./dist/incident.wasm",
        "assets": "./dist/assets-manifest.json",
        "sounds": sounds,
    }, fh, indent=2)

print(f"   {len(fs_files)} FS files, {sum(len(v) for v in sounds.values())} sound files in {len(sounds)} categories")
PY

echo ">> done. serve the 'web' directory with cross-origin isolation, e.g.:"
echo "     python3 $here/serve.py"
