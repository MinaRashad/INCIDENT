use crate::game_state::endings::Ending;
use crate::{data::player, game_state::endings};
use crate::{data, menu_components};
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
    End(endings::Ending)
}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct DocumentPath(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum EventType {
    StartGame,
    OnPathOpen(PathBuf),
    OnDialogueNode(String),
    OnDialogueChoice(String),
    OnContradiction(DocumentPath, DocumentPath, u64),
    EndGame(Ending)
}

impl EventType {
    /// The events are structured in the following format:
    /// event_name;arg1;arg2;..etc
    /// A possible potential problem is if the path for opening
    /// contains a ; otherwise, everything is controlled
    pub fn from_str(s: &str) -> Self{
        let parts: Vec<&str> = s.split(';').filter(|s| !s.is_empty()).collect();
        match parts.as_slice() {
                ["path", path] => EventType::OnPathOpen(PathBuf::from(path)),
                ["dialogue", id] => EventType::OnDialogueNode(id.to_string()),
                ["choose", id] => EventType::OnDialogueChoice(id.to_string()),
                ["start"] => EventType::StartGame,
                ["end",ending] => EventType::EndGame(
                        Ending::from_str(*ending)
                        .unwrap_or(Ending::Refusal)
                    ),
                ["contradiction", doc1, doc2, tag_id]
                    => {
                        EventType::OnContradiction(
                            DocumentPath(doc1.to_string()), 
                            DocumentPath(doc2.to_string()),
                            tag_id.parse().unwrap_or_default()
                        )
                    }
                _ => panic!("Unparsable event type: {s}")
            }
    }
    pub fn to_str(&self) -> String {
        match self {
            EventType::OnPathOpen(path) => format!(";path;{};", path.display()),
            EventType::OnDialogueNode(id) => format!(";dialogue;{id};"),
            EventType::OnDialogueChoice(id) => format!(";choose;{id};"),
            EventType::OnContradiction(
                    DocumentPath(doc1),
                    DocumentPath(doc2),
                    contradicting_tag
                 ) => format!("contradiction;{};{};{};", 
                    doc1, doc2, contradicting_tag
                ),
            EventType::StartGame => ";start;".to_string(),
            EventType::EndGame(ending) => format!(";end;{};", ending.to_str()),
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
            Effect::End(ending) => endings::end(*ending)
        }
    }
}

/// in-memory static data for the effects
pub fn init_events()
-> HashMap<EventType, Effect> 
{
    let mut map : HashMap<EventType, Effect> = HashMap::new();

    map.insert(EventType::OnDialogueChoice("declined".to_string()),
     Effect::End(Ending::Refusal));

    map.insert(EventType::OnDialogueChoice("notread".to_string()),
     Effect::End(Ending::FailedInterview));

    map.insert(EventType::OnDialogueChoice("passinterview".to_string()),
     Effect::End(Ending::Hired));
     
    return map;
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
-> Vec<(i32,EventType)>
{
    let result  = data::METADATA_DB.with(
        |db| 
        -> Result<Vec<(i32,EventType)>, rusqlite::Error>{
        let conn = db.get().expect("Database not initialized");

        let mut statement = conn
            .prepare("SELECT id,name 
            FROM
            history 
            WHERE
            processed_at IS NULL
            ORDER BY created_at")
            .expect("Unable to create SQL statement");
        
        let events :Vec<(i32,EventType)> =
            statement.query_map([],
            |row|    
            Ok(
                (
                row.get::<usize,i32>(0)?,
                EventType::from_str(row.get::<usize,String>(1)?.as_str()))
                )
        )?
        .filter(|row| row.is_ok())
        .map(|row| row.unwrap())
        .collect();

        Ok(events)   
    });

    result.unwrap_or(vec![])
}

fn process_event(id:i64) -> Result<(), rusqlite::Error>{
    let _ = data::METADATA_DB.with(|db|{
        let conn = db.get().expect("Database not initialized");
        let timestamp = std::time::SystemTime::now();
        let timestamp = timestamp
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() )
                .unwrap_or(0);


        conn.execute(
            "UPDATE history SET processed_at = ?1 WHERE id = ?2", 
            (timestamp as i64, id))
    })?;

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

fn run_event_master()->Option<()>{
    
    data::init_db();
    let event_map = init_events();
    loop {
        // get the un processed history
        let unprocessed = get_unprocessed_history();

        // loop through it
        for (id,event) in unprocessed{            
            // process each event

            // Important NOTE:
            // mark as processed FIRST
            // I initially had this after processing but some events
            // like end, kills the entire process so it never gets
            // marked as process and gets processed each time from
            // now on
            // the potential problem is if the effect failed

            let result = process_event(id as i64);


            if let Some(effect) = event_map.get(&event){
                effect.activate();
            }
            
        }

        std::thread::sleep(Duration::from_secs(1));       
    }
}