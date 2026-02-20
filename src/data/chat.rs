use ratatui::widgets::ListState;

use std::{
    collections::HashMap, 
    fs::File, 
    io::Error, 
    path::PathBuf, 
    thread
};

use serde::{Deserialize, Serialize};
use serde_json;
use crate::menu_components;

const CHATS_PATH: &str = "assets/Chat/dialogue.json";

/// Chatlog is the current seen chatlog
pub struct ChatLog{
    pub sender : NPC,
    pub messages : Vec<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum NPC {
    Marcus,
    Sarah,
    Jessica,
    David,
    Mike,
    Rodriguez,
    Elizabeth,
    Robert,
    Susan,
    Jennifer
}

impl NPC{
    pub const ALL: [NPC;10] = [NPC::David, NPC::Elizabeth, NPC::Jennifer, NPC::Jessica, NPC::Marcus,
                NPC::Mike, NPC::Robert, NPC::Rodriguez, NPC::Sarah, NPC::Susan];

    pub fn name(&self) -> &str {
        match self {
            NPC::Marcus => "Marcus",
            NPC::Sarah => "Sarah",
            NPC::Jessica => "Jessica",
            NPC::David => "David",
            NPC::Mike => "Mike",
            NPC::Rodriguez => "Rodriguez",
            NPC::Elizabeth => "Elizabeth",
            NPC::Robert => "Robert",
            NPC::Susan => "Susan",
            NPC::Jennifer => "Jennifer",
        }
    }
}

#[derive(Debug, Default,Deserialize, Serialize)]
pub struct Message{
    pub is_recieved: bool,
    pub content:String
}

#[derive(Debug,Default)]
pub struct ChatAppState{
    pub current_chat_selection:ListState,
    pub current_choice_selection:usize,
    pub choices: Vec<Choice>,
    pub chat_scroll:usize,
    pub running:bool
}


#[derive(Debug, Default,Deserialize, Serialize)]
pub struct Choice {
    pub text: String,           // display text
    pub next_dialogue: String,      // where this goes in dialogue tree
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DialogueNode{
    text: Option<String>,
    options: Vec<Choice>,
    conditions: Vec<String>,    // custom flags checked before allowing it
    events: Vec<String>,         // custome flags emitted
    next_dialogue:Option<String>
}
pub type DialogueTree =  HashMap<NPC, HashMap<String, DialogueNode>>;



pub fn spawn_chat_master(){
    thread::spawn(run_chat_master);
}

/// # run_chat_master
/// this function is runs by the chat master
/// It does the following in an infinite loop:
/// - Checks if there are new conditions unlocked
/// - If there is, checks all npcs current dialogue 
fn run_chat_master(){
    let chats = read_dialogue_data();

    println!("Spawned the chat master!");
    println!("The chat master says:");
    match chats {
        Ok(chats) => println!("{chats:?}"),
        Err(err) => println!("{err}")       
    }

    menu_components::wait_for_input();

}

fn read_dialogue_data()-> Result<DialogueTree, Error>{

    let chats_json_path = PathBuf::from(CHATS_PATH);
    let chats_file = File::open(chats_json_path)?;    

    // now we have the chat file, we just need to parse
    // it as JSON

    let json: DialogueTree = serde_json::from_reader(chats_file)?;



    Ok(json)
}