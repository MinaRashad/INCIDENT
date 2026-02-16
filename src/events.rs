use crate::data::player;
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::OnceLock};

#[derive(Debug)]
pub enum Effect{
    IncreaseClearance,
    DecreaseClearance,
    SetClearance(u32)
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

pub static ON_OPEN:OnceLock< HashMap<&OsStr, Effect> > = OnceLock::new() ;

pub fn init_events() {
    let mut map = HashMap::new();

    map.insert(OsStr::new("Det. Rodriguez"), Effect::IncreaseClearance);

    let _ = ON_OPEN.set(map);
}