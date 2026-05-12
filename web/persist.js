// IndexedDB-backed persistence for the game's SQLite database (`main.db`).
//
// On the desktop the game keeps `main.db` (and the WAL/SHM siblings) on disk.
// In the browser the WASI filesystem is in-memory, so we snapshot the relevant
// files to IndexedDB whenever the worker reports them dirty, and restore them
// on startup. Keyed by a fixed slot name so multiple browser windows of the
// same game share one save (mirrors several OS processes sharing main.db).

const DB_NAME = "incident-save";
const STORE = "files";

function open() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE);
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

// Returns { "main.db": Uint8Array, ... } or {} if nothing saved yet.
export async function loadSave() {
  console.log("[persist] Loading save from IndexedDB...");
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readonly");
    const req = tx.objectStore(STORE).get("snapshot");
    req.onsuccess = () => {
      console.log("[persist] Save loaded:", req.result ? Object.keys(req.result) : "none");
      resolve(req.result || {});
    };
    req.onerror = () => {
      console.error("[persist] Load failed:", req.error);
      reject(req.error);
    };
  });
}

// `files` is { "main.db": Uint8Array, ... }.
export async function writeSave(files) {
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(files, "snapshot");
    tx.oncomplete = () => {
      console.log("[persist] Write complete");
      resolve();
    };
    tx.onerror = () => {
      console.error("[persist] Write failed:", tx.error);
      reject(tx.error);
    };
  });
}

export async function clearSave() {
  const db = await open();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).delete("snapshot");
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}
