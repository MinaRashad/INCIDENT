

pub const OS_LOGO_PATH:&str = "assets/images/OS_logo.png";

use rusqlite::params;
use rusqlite::OptionalExtension;
use std::path::PathBuf;
use serde::Serialize;

use crate::data::METADATA_DB;



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


pub fn metadata(entry: &Entry) -> Metadata {

    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path = entry.path.to_string_lossy().replace("\\", "/");
        conn.query_row(
            "SELECT access_level, password, opened
             FROM metadata
             WHERE path = ?1",
            [path],
            |row| {
                Ok(Metadata {
                    access_level: row.get::<_, Option<i64>>(0)?.map(|v| v as usize),
                    password: row.get(1)?,
                    opened: row.get::<_, i64>(2)? != 0,
                })
            },
        )
        .optional()
        .expect("Failed to query metadata")
    }).unwrap_or(Metadata::new())
}

pub fn update_metadata(entry: &Entry, field: MetadataField)  {

    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path = entry.path.to_string_lossy().replace("\\", "/");;
        let col = field.column_name();
        let val = field.sqlite_value();

        let sql = format!(
            "INSERT INTO metadata (path, {col}) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET {col} = excluded.{col}"
        );

        conn.execute(&sql, params![path, val])
            .expect("Failed to update metadata");
    });
}
