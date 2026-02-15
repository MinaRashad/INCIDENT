pub mod player;
pub mod chat;
pub mod docs;

use std::sync::OnceLock;
use rusqlite::Connection;
use serde::Serialize;
use std::{ fs, path::{Path, PathBuf}};
/*
    Here I will define the model of our data
    This is similar to 3N standards
*/

// chatlogs

thread_local! {
    pub static METADATA_DB: OnceLock<Connection> = OnceLock::new();
}

const DEFAULT_METADATA_FILE: &str = "default.sql";
const MAIN_METADATA_FILE:&str = "main.db";





// Documents

pub struct Password{
    content:String
}

pub struct ImageDoc(pub PathBuf);

impl ImageDoc {
    // If you were creating them like this before: ImageDoc::Image(path)
    // You can keep doing that by adding a constructor:
    pub fn image(path: PathBuf) -> Self {
        ImageDoc(path)
    }

    // If you were matching on them before:
    // match doc { ImageDoc::Image(p) => ... }
    // You can update the match to look like this:
    pub fn get_path(&self) -> &PathBuf {
        &self.0
    }
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
