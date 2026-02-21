use crate::data::{chat::NPC, player};
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

#[derive(Debug)]
pub enum Effect{
    IncreaseClearance,
    DecreaseClearance,
    SetClearance(u32)
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
        }
    }
}

pub static ON_EVENT:OnceLock< HashMap<EventType, Effect> > = OnceLock::new() ;

pub fn init_events() {
    let mut map = HashMap::new();


    let _ = ON_EVENT.set(map);
}