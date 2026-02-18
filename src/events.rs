use crate::data::{chat::NPC, player};
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

#[derive(Debug)]
pub enum Effect{
    IncreaseClearance,
    DecreaseClearance,
    SetClearance(u32)
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    OnChatWith(NPC),
    OnPathOpen(PathBuf)
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

    map.insert(EventType::OnChatWith(NPC::Rodriguez), Effect::IncreaseClearance);

    let _ = ON_EVENT.set(map);
}