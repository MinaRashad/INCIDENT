

pub const OS_LOGO_PATH:&str = "assets/images/OS_logo.png";
use serde::Serialize;
use std::{ fs, path::{Path, PathBuf}};
use rusqlite::params;
use std::sync::OnceLock;
use rusqlite::Connection;


const DEFAULT_METADATA_FILE: &str = "default.sql";
const MAIN_METADATA_FILE:&str = "main.db";

thread_local! {
    static METADATA_DB: OnceLock<Connection> = OnceLock::new();
}

// metadata for files
#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize)]
pub struct Entry{
    pub path:PathBuf
}
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct Metadata{
    pub access_level:Option<usize>,
    pub password: Option<String>,
    pub opened:bool
}

pub enum MetadataField {
    AccessLevel(usize),
    Password(String),
    Opened(bool),
}

impl MetadataField {
    fn column_name(&self) -> &'static str {
        match self {
            MetadataField::AccessLevel(_) => "access_level",
            MetadataField::Password(_) => "password",
            MetadataField::Opened(_) => "opened",
        }
    }

    fn sqlite_value(&self) -> rusqlite::types::Value {
        match self {
            MetadataField::AccessLevel(v) => rusqlite::types::Value::Integer(*v as i64),
            MetadataField::Password(s) => rusqlite::types::Value::Text(s.clone()),
            MetadataField::Opened(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        }
    }
}

impl Metadata {
    pub fn new()->Metadata{
        Metadata{
            access_level:None,
            password:None,
            opened:false
        }
    }    
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Tag{
    name:String
}

pub fn init_metadata() {

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

pub fn metadata(entry: &Entry) -> Option<Metadata> {
    use rusqlite::OptionalExtension;

    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path = entry.path.to_string_lossy();

        conn.query_row(
            "SELECT access_level, password, opened
             FROM metadata
             WHERE path = ?1",
            [path.as_ref()],
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
    })
}

pub fn update_metadata(entry: &Entry, field: MetadataField)  {

    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path = entry.path.to_string_lossy();
        let col = field.column_name();
        let val = field.sqlite_value();

        let sql = format!(
            "INSERT INTO metadata (path, {col}) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET {col} = excluded.{col}"
        );

        conn.execute(&sql, params![path.as_ref(), val])
            .expect("Failed to update metadata");
    });
}
