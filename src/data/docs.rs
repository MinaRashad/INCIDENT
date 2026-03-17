

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
/// Used mainly for contraditions
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Default)]
pub struct Tag{
    pub id: u32,
    pub value: String // We might need a more general way for this
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


// another type of documents that we will add is player-made documents
// for now, this includes:
// - Contradictions, These will be used as means to progress in the game
// - Notes, Theses are player generated (risky) and only used to help them feel like a detective
//          They can use it to store relevant facts or things they found out

pub enum PlayerDocument{
    Note(Note),
    Contradiction(Contradiction)
}

pub fn get_player_documents() -> Vec<PlayerDocument> {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let mut stmt = conn.prepare("SELECT doc_id, doc_table FROM user_docs ORDER BY id ASC").expect("Failed to prepare statement");
        
        let docs_iter = stmt.query_map([], |row| {
            let doc_id: i64 = row.get(0)?;
            let doc_table: String = row.get(1)?;
            Ok((doc_id, doc_table))
        }).expect("Failed to query user_docs");

        let mut results = Vec::new();
        for doc in docs_iter {
            let (doc_id, doc_table) = doc.expect("Failed to parse user_doc");
            if doc_table == "notes" {
                let mut note_stmt = conn.prepare("SELECT title, content, doc_path FROM notes WHERE id = ?1").expect("Failed to prepare note statement");
                let note = note_stmt.query_row([doc_id], |row| {
                    let path_str: Option<String> = row.get(2)?;
                    Ok(Note {
                        title: row.get(0)?,
                        content: row.get(1)?,
                        path: path_str.map(PathBuf::from),
                    })
                }).expect("Note id not found");
                results.push(PlayerDocument::Note(note));
            } else if doc_table == "contradictions" {
                 let mut contra_stmt = conn.prepare(
                    "SELECT doc_path_a, doc_path_b, contradictions.tag_id, tags.name, metadata_tags.value 
                     FROM contradictions 
                     JOIN tags ON contradictions.tag_id = tags.id
                     JOIN metadata_tags ON contradictions.doc_path_a = metadata_tags.path AND contradictions.tag_id = metadata_tags.tag_id
                     WHERE contradictions.id = ?1"
                ).expect("Failed to prepare contradiction statement");
                
                let contradiction = contra_stmt.query_row([doc_id], |row| {
                    Ok(Contradiction {
                        doc1: PathBuf::from(row.get::<_, String>(0)?),
                        doc2: PathBuf::from(row.get::<_, String>(1)?),
                        disagree_on: Tag {
                            id: row.get(2)?,
                            value: row.get(4)?,
                        },
                    })
                }).expect("Contradiction id not found");
                results.push(PlayerDocument::Contradiction(contradiction));
            }
        }
        results
    })
}

pub struct Note {
    pub path:Option<PathBuf>,
    pub title:String,
    pub content:String
}

pub struct Contradiction{
    pub doc1: PathBuf,
    pub doc2: PathBuf,
    pub disagree_on:Tag
}


pub fn get_tags_of(entry: &Entry) -> Vec<Tag> {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path = entry.path.to_string_lossy().replace("\\", "/");

        let mut stmt = conn.prepare(
            "SELECT tag_id, value FROM metadata_tags WHERE path = ?1"
        ).ok()?;

        let tags: Vec<Tag> = stmt.query_map([path], |row| {
            Ok(Tag {
                id: row.get::<_, u32>(0)?,
                value: row.get::<_, String>(1)?,
            })
        }).ok()?
        .flatten()
        .collect();

        Some(tags)
    }).unwrap_or(vec![])
}

pub fn get_tag_name(tag_id: u32) -> String {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        conn.query_row(
            "SELECT name FROM tags WHERE id = ?1",
            [tag_id],
            |row| row.get::<_, String>(0),
        )
        .expect("Tag id not found — wrong tag id passed to get_tag_name")
    })
}

pub fn add_note(note: Note) {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path = note.path.map(|p| p.to_string_lossy().replace("\\", "/"));

        conn.execute(
            "INSERT INTO notes (title, content, doc_path) VALUES (?1, ?2, ?3)",
            params![note.title, note.content, path],
        )
        .expect("Failed to insert note");

        let note_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO user_docs (doc_id, doc_table) VALUES (?1, 'notes')",
            params![note_id],
        )
        .expect("Failed to insert into user_docs");
    });
}

pub fn get_notes() -> Vec<Note> {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let mut stmt = conn.prepare("SELECT title, content, doc_path FROM notes").expect("Failed to prepare statement");
        let note_iter = stmt.query_map([], |row| {
            let path_str: Option<String> = row.get(2)?;
            Ok(Note {
                title: row.get(0)?,
                content: row.get(1)?,
                path: path_str.map(PathBuf::from),
            })
        }).expect("Failed to query notes");

        note_iter.map(|n| n.expect("Failed to parse note")).collect()
    })
}

pub fn get_contradictions() -> Vec<Contradiction> {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let mut stmt = conn.prepare(
            "SELECT doc_path_a, doc_path_b, contradictions.tag_id, tags.name, metadata_tags.value 
             FROM contradictions 
             JOIN tags ON contradictions.tag_id = tags.id
             JOIN metadata_tags ON contradictions.doc_path_a = metadata_tags.path AND contradictions.tag_id = metadata_tags.tag_id"
        ).expect("Failed to prepare statement");

        let contradiction_iter = stmt.query_map([], |row| {
            Ok(Contradiction {
                doc1: PathBuf::from(row.get::<_, String>(0)?),
                doc2: PathBuf::from(row.get::<_, String>(1)?),
                disagree_on: Tag {
                    id: row.get(2)?,
                    value: row.get(4)?,
                },
            })
        }).expect("Failed to query contradictions");

        contradiction_iter.map(|c| c.expect("Failed to parse contradiction")).collect()
    })
}

pub fn add_contradiction(contradiction: Contradiction) {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path_a = contradiction.doc1.to_string_lossy().replace("\\", "/");
        let path_b = contradiction.doc2.to_string_lossy().replace("\\", "/");
        let tag_id = contradiction.disagree_on.id;

        conn.execute(
            "INSERT INTO contradictions (doc_path_a, doc_path_b, tag_id) VALUES (?1, ?2, ?3)",
            params![path_a, path_b, tag_id],
        )
        .expect("Failed to insert contradiction");

        let contradiction_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO user_docs (doc_id, doc_table) VALUES (?1, 'contradictions')",
            params![contradiction_id],
        )
        .expect("Failed to insert into user_docs");
    });
}

pub fn contradiction_exists(contradiction: &Contradiction) -> bool {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");
        let path_a = contradiction.doc1.to_string_lossy().replace("\\", "/");
        let path_b = contradiction.doc2.to_string_lossy().replace("\\", "/");
        let tag_id = contradiction.disagree_on.id;

        conn.query_row(
            "SELECT 1 FROM contradictions
             WHERE tag_id = ?3
             AND ((doc_path_a = ?1 AND doc_path_b = ?2)
               OR (doc_path_a = ?2 AND doc_path_b = ?1))",
            params![path_a, path_b, tag_id],
            |_| Ok(()),
        )
        .optional()
        .expect("Failed to query contradictions")
        .is_some()
    })
}