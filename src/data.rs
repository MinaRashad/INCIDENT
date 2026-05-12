pub mod player;
pub mod chat;
pub mod docs;

use std::sync::OnceLock;
use rusqlite::Connection;
use std::{ fs, path::{Path, PathBuf}};
use sha2::{Sha256, Digest};



// / Thread-local database connection for file metadata
// / Each thread gets its own connection instance via OnceLock
// / Initialized once per thread by init_db()
thread_local! {
    pub static METADATA_DB: OnceLock<Connection> = OnceLock::new();
}

/// SQL schema file containing the default database structure
const DEFAULT_METADATA_FILE: &str = "default.sql";

/// Main SQLite database file for storing file metadata
const MAIN_METADATA_FILE:&str = "main.db";




/// Initializes the metadata database
/// Creates a new database file if it doesn't exist, using the schema from default.sql
/// Opens an existing database if the file already exists
/// Sets up the thread-local database connection
/// 
/// # Panics
/// Panics if the database cannot be opened, schema cannot be read,
/// or if the database has already been initialized on this thread
pub fn init_db() {
    let exists = Path::new(MAIN_METADATA_FILE).exists();
    let len = fs::metadata(MAIN_METADATA_FILE).map(|m| m.len()).unwrap_or(0);

    // this function creates/opens the database
    let conn = Connection::open(MAIN_METADATA_FILE)
        .expect("Failed to open main.db");

    // A more robust check: does the metadata table exist?
    let is_new = !exists || len == 0 || (|| -> Result<bool, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='metadata'")?;
        Ok(!stmt.exists([])?)
    })().unwrap_or(true);


    // Optimized pragmas for wasm environment.
    #[cfg(target_arch = "wasm32")]
    {
        let _ = conn.execute("PRAGMA journal_mode=DELETE;", []);
        let _ = conn.execute("PRAGMA locking_mode=EXCLUSIVE;", []);
        let _ = conn.execute("PRAGMA temp_store=MEMORY;", []);
        let _ = conn.execute("PRAGMA synchronous=OFF;", []);
    }

    if is_new {
        let schema = fs::read_to_string(DEFAULT_METADATA_FILE)
            .expect("Failed to read default.sql");

        conn.execute_batch(&schema)
            .expect("Failed to initialize database");
    }

    // Final enforcement of journal mode after schema (which might have had its own pragmas)
    #[cfg(target_arch = "wasm32")]
    {
        let mode: String = conn.query_row("PRAGMA journal_mode=DELETE", [], |r| r.get(0)).unwrap_or_default();
        eprintln!("[game] Final journal_mode: {}", mode);
    }

    METADATA_DB.with(|db| {
        db.set(conn).expect("Already initialized");
    });
}




/// Computes SHA-256 hash of the input string
/// Returns the hash as a hexadecimal string
/// Used for password hashing and verification
/// 
/// # Arguments
/// * `input` - The string to hash
/// 
/// # Returns
/// Hexadecimal representation of the SHA-256 hash
pub fn sha256(input:String) -> String{
    let mut hasher = Sha256::new();
    hasher.update(input);
    let hash:String= hasher
        .finalize().iter().map(|b| format!("{:02x}", b))
        .collect();

    hash
}
