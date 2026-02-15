
use std::sync::OnceLock;
use whoami;

use crate::data::METADATA_DB;

#[derive(Clone, Debug)]
pub struct Player{
    name:String,
    access_level:usize
}


pub fn init_player(){
    let user = 
            whoami::realname()
            .or(whoami::username());

    let user = match user{
        Ok(name)=>name,
        Err(_)=>"You".to_string()
    };
    METADATA_DB.with(|db|{
        let conn = db.get().expect("Unable to get db connection");
        conn.execute(
            "INSERT INTO player (id, name, access_level)
             VALUES (0, ?, 0)
             ON CONFLICT(id) DO UPDATE SET
                 name=excluded.name,
                 access_level=excluded.access_level;",
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
