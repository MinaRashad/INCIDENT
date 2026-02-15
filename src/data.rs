pub mod player;
pub mod chat;
pub mod docs;

use std::sync::OnceLock;
use rusqlite::Connection;
use serde::Serialize;
use std::{ fs, path::{Path, PathBuf}};
use sha2::{self, Digest};


/// Thread-local database connection for file metadata
/// Each thread gets its own connection instance via OnceLock
/// Initialized once per thread by init_db()
thread_local! {
    pub static METADATA_DB: OnceLock<Connection> = OnceLock::new();
}

/// SQL schema file containing the default database structure
const DEFAULT_METADATA_FILE: &str = "default.sql";

/// Main SQLite database file for storing file metadata
const MAIN_METADATA_FILE:&str = "main.db";


// Documents

/// Represents a password string
/// Wraps the password content for type safety
pub struct Password{
    content:String
}

/// Represents an image document with its file path
/// Tuple struct containing the path to the image file
pub struct ImageDoc(pub PathBuf);

impl ImageDoc {
    /// Creates a new ImageDoc from a PathBuf
    /// Convenience constructor for creating image document references
    pub fn image(path: PathBuf) -> Self {
        ImageDoc(path)
    }

    /// Returns a reference to the image file path
    /// Provides access to the wrapped PathBuf
    pub fn get_path(&self) -> &PathBuf {
        &self.0
    }
}


/// Represents a file system entry (file or directory)
/// Used as a key to look up metadata in the database
#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize)]
pub struct Entry{
    pub path:PathBuf
}

/// Metadata associated with a file or directory
/// Tracks access control, password protection, and open status
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct Metadata{
    /// Required clearance level to access this entry (None = no restriction)
    pub access_level:Option<usize>,
    /// SHA-256 hash of the password if password-protected (None = no password)
    pub password: Option<String>,
    /// Whether this entry has been opened/accessed by the player
    pub opened:bool
}

/// Represents a single metadata field that can be updated in the database
/// Used for constructing database update queries
pub enum MetadataField {
    /// Access level requirement
    AccessLevel(usize),
    /// Password hash
    Password(String),
    /// Opened status
    Opened(bool),
}

impl MetadataField {
    /// Returns the database column name for this field
    fn column_name(&self) -> &'static str {
        match self {
            MetadataField::AccessLevel(_) => "access_level",
            MetadataField::Password(_) => "password",
            MetadataField::Opened(_) => "opened",
        }
    }

    /// Converts the field value to a SQLite-compatible value
    /// Handles type conversions: bool to integer, strings and numbers as-is
    fn sqlite_value(&self) -> rusqlite::types::Value {
        match self {
            MetadataField::AccessLevel(v) => rusqlite::types::Value::Integer(*v as i64),
            MetadataField::Password(s) => rusqlite::types::Value::Text(s.clone()),
            MetadataField::Opened(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        }
    }
}

impl Metadata {
    /// Creates a new Metadata instance with default values
    /// No access restrictions, no password, not yet opened
    pub fn new()->Metadata{
        Metadata{
            access_level:None,
            password:None,
            opened:false
        }
    }    
}

/// Represents a tag that can be associated with entries
/// Used for categorizing or labeling files and directories
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Tag{
    name:String
}

/// Initializes the metadata database
/// Creates a new database file if it doesn't exist, using the schema from default.sql
/// Opens an existing database if the file already exists
/// Sets up the thread-local database connection
/// 
/// # Panics
/// Panics if the database cannot be opened, schema cannot be read,
/// or if the database has already been initialized on this thread
pub fn init_db() {

    let is_new = !Path::new(MAIN_METADATA_FILE).exists();

    // this function creates/opens the database
    let conn = Connection::open(MAIN_METADATA_FILE)
        .expect("Failed to open main.db");

    if is_new {
        let schema = fs::read_to_string(DEFAULT_METADATA_FILE)
            .expect("Failed to read default.sql");

        conn.execute_batch(&schema)
            .expect("Failed to initialize database");
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
pub fn sha256(input:&str)->String{
    let hasher = sha2::Sha256::new();
    todo!()
}