#[cfg(not(target_arch = "wasm32"))]
use whoami;

use crate::data::METADATA_DB;

/// Best-effort current user name. On wasm there's no OS user, so the browser
/// host can supply one via the `INCIDENT_PLAYER_NAME` env var; otherwise "You".
#[cfg(not(target_arch = "wasm32"))]
fn current_username() -> String {
    match whoami::realname().or(whoami::username()) {
        Ok(name) => name,
        Err(_) => "You".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
fn current_username() -> String {
    std::env::var("INCIDENT_PLAYER_NAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "You".to_string())
}

pub fn init_player(){
    let user = current_username();
    METADATA_DB.with(|db|{
        let conn = db.get().expect("Unable to get db connection");
        conn.execute(
            "INSERT INTO player (id, name)
             VALUES (0, ?)
             ON CONFLICT(id) DO UPDATE SET
                 name=excluded.name;",
            [user],
        )
        .expect("Failed to insert or update player");
    })
}

pub fn get_access_level() -> Option<i32> {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Unable to get db connection");
        conn.query_row(
            "SELECT access_level FROM player WHERE id = 0",
            [],
            |row| row.get(0),
        ).ok()
    })
}

// Set the player's access level
pub fn set_access_level(level: i32) {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Unable to get db connection");
        conn.execute(
            "UPDATE player SET access_level = ? WHERE id = 0",
            [level],
        ).expect("Failed to update access level");
    });
}

// Get the player's name
pub fn get_player_name() -> Option<String> {
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Unable to get db connection");
        conn.query_row(
            "SELECT name FROM player WHERE id = 0",
            [],
            |row| row.get(0),
        ).ok()
    })
}

pub fn hire(){
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Unable to get db connection");
        conn.execute(
            "UPDATE player SET hired = 1 WHERE id = 0",
            [],
        ).expect("Failed to hire player");
    });
}

pub fn fire(){
    METADATA_DB.with(|db| {
        let conn = db.get().expect("Unable to get db connection");
        conn.execute(
            "UPDATE player SET hired = 0 WHERE id = 0",
            [],
        ).expect("Failed to fire player");
    });
}


pub fn is_hired() -> bool{
    let hired = METADATA_DB.with(|db| 
        -> Option<i32>{
        let conn = db.get().expect("Unable to get db connection");
        conn.query_row(
            "SELECT hired FROM player WHERE id = 0",
            [],
            |row| row.get(0),
        ).ok()
    });
    if let Some(val) = hired && val != 0{
        return true;
    }else{
        return false;
    }
}