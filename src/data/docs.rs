

pub const OS_LOGO_PATH:&str = "assets/images/OS_logo.png";

use rusqlite::params;
use rusqlite::OptionalExtension;
use crate::data::{Entry, Tag, Metadata, MetadataField};

use crate::data::METADATA_DB;





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
