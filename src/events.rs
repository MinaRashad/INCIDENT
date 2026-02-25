use crate::{data::player, game_state::endings};
use crate::data;
use env_logger::fmt::Timestamp;
use log::error;


use core::time;
use std::time::{Duration, UNIX_EPOCH};
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

#[derive(Debug)]
pub enum Effect{
    IncreaseClearance,
    DecreaseClearance,
    SetClearance(u32),
    Hire,
    Fire,
    End
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum EventType {
    StartGame,
    OnPathOpen(PathBuf),
    OnDialogueNode(String),
    OnDialogueChoice(String)
}

impl EventType {
    pub fn from_str(s: &str) -> Self{
    let parts: Vec<&str> = s.split(';').filter(|s| !s.is_empty()).collect();
    match parts.as_slice() {
            ["path", path] => EventType::OnPathOpen(PathBuf::from(path)),
            ["dialogue", id] => EventType::OnDialogueNode(id.to_string()),
            ["choose", id] => EventType::OnDialogueChoice(id.to_string()),
            ["start"] => EventType::StartGame,
            _ => panic!("Unparsable event type: {s}")
        }
    }
}

impl Effect {
    pub fn activate(&self){
        match &self {
            Effect::IncreaseClearance => player::set_access_level(
                player::get_access_level().unwrap_or(0) + 1
            ),
            Effect::DecreaseClearance => player::set_access_level(
                player::get_access_level().unwrap_or(0) - 1
            ),
            Effect::SetClearance(clearance) => player::set_access_level(*clearance as i32),
            Effect::Hire => player::hire(),
            Effect::Fire => player::fire(),
            Effect::End => endings::end()
        }
    }
}

pub static ON_EVENT:OnceLock< HashMap<EventType, Effect> > = OnceLock::new() ;

/// in-memory static data for the effects
pub fn init_events() {
    let mut map = HashMap::new();


    let _ = ON_EVENT.set(map);
}



pub fn get_history(after: u32)
-> Vec<EventType>
{
    let result  = data::METADATA_DB.with(
        |db| 
        -> Result<Vec<EventType>, rusqlite::Error>{
        let conn = db.get().expect("Database not initialized");

        let mut statement = conn
            .prepare("SELECT name 
            FROM
            history 
            WHERE
            created_at > ?1
            ORDER BY created_at")
            .expect("Unable to create SQL statement");
        
        let events :Vec<EventType> =
            statement.query_map((after,),
            |row|    
            Ok(EventType::from_str(row.get::<usize,String>(0)?.as_str()))
        )?
        .filter(|row| row.is_ok())
        .map(|row| row.unwrap())
        .collect();

        Ok(events)   
    });

    result.unwrap_or(vec![])
}

fn get_unprocessed_history()
-> Vec<EventType>
{
    let result  = data::METADATA_DB.with(
        |db| 
        -> Result<Vec<EventType>, rusqlite::Error>{
        let conn = db.get().expect("Database not initialized");

        let mut statement = conn
            .prepare("SELECT name 
            FROM
            history 
            WHERE
            processed_at IS NULL
            ORDER BY created_at")
            .expect("Unable to create SQL statement");
        
        let events :Vec<EventType> =
            statement.query_map([],
            |row|    
            Ok(EventType::from_str(row.get::<usize,String>(0)?.as_str()))
        )?
        .filter(|row| row.is_ok())
        .map(|row| row.unwrap())
        .collect();

        Ok(events)   
    });

    result.unwrap_or(vec![])
}

fn process_event(id:i64) -> Result<(), rusqlite::Error>{
    let result = data::METADATA_DB.with(|db|{
        let conn = db.get().expect("Database not initialized");
        let timestamp = std::time::SystemTime::now();
        let timestamp = timestamp
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() )
                .unwrap_or(0);


        conn.execute(
            "UPDATE history (processed_at) VALUES (?1) WHERE id=?2", 
            (timestamp as i64, id))
    });

    Ok(())
}

pub fn add_event(event_tag:String){
    let result = data::METADATA_DB.with(|db|{
        let conn = db.get().expect("Database not initialized");

        conn.execute(
            "INSERT INTO history (name) VALUES (?1)", 
            (event_tag,))
    });

    if result.is_err(){
        error!("Failed to send message")
    }
}


/// Spawns the event processer
/// it checks the game history and activates relevant effects
pub fn spawn_all_seeing_eye(){
    std::thread::spawn(run_event_master);
}

fn run_event_master(){
    loop {
        // get the un processed history

        // loop through it

        // process each event

        // mark as processed



        std::thread::sleep(Duration::from_secs(1));       
    }
}