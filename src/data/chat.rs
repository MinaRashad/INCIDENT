use log::error;
use ratatui::widgets::ListState;

use std::{
    collections::HashMap, 
    fs::File, 
    io::Error, 
    path::PathBuf, 
    thread, vec
};

use serde::{Deserialize, Serialize};
use serde_json;
use crate::{data, menu_components};

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
#[serde(default)]
pub struct Choice {
    pub text: String,           // display text
    pub next_dialogue: String,      // where this goes in dialogue tree
    pub conditions: Vec<String>,
    pub events: Vec<String>
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

enum Contact_Char{
    Player,
    NPC(NPC)
}

impl Contact_Char{
    fn to_string(&self) ->String{
        match self {
            Contact_Char::Player => "Player".to_string(),
            Contact_Char::NPC(npc) => npc.name().to_string()          
        }
    }
}

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

     // TEST: send a message from Marcus to Player
    send_message(Contact_Char::NPC(NPC::Marcus), Contact_Char::Player, "Hey, this is a test message.".to_string());
    println!("Sent a message from Marcus to Player");

    // TEST: send a message from Player to Marcus
    send_message(Contact_Char::Player, Contact_Char::NPC(NPC::Marcus), "Got it, thanks Marcus.".to_string());
    println!("Sent a message from Player to Marcus");

    // TEST: check messages for Marcus (should see both)
    let messages = check_messages(0, NPC::Marcus);
    println!("Messages for Marcus ({} found):", messages.len());
    for msg in &messages {
        println!("  is_received: {} | content: {}", msg.is_recieved, msg.content);
    }

}

fn read_dialogue_data()-> Result<DialogueTree, Error>{

    let chats_json_path = PathBuf::from(CHATS_PATH);
    let chats_file = File::open(chats_json_path)?; 

    data::init_db();   

    // now we have the chat file, we just need to parse
    // it as JSON

    let json: DialogueTree = serde_json::from_reader(chats_file)?;
    Ok(json)
}


fn send_message(from: Contact_Char, to:Contact_Char, Message:String)

{
    let result = data::METADATA_DB.with(|db|{
        let conn = db.get().expect("Database not initialized");

        conn.execute(
            "INSERT INTO messages (sender, receiver, content) VALUES (?1, ?2, ?3)", 
            (from.to_string(), to.to_string(), Message))
    });

    if result.is_err(){
        error!("Failed to send message")
    }
}

fn check_messages(after: u32, npc:NPC) 
-> Vec<Message>{
    
    let result  = data::METADATA_DB.with(
        |db| 
        -> Result<Vec<Message>, rusqlite::Error>{
        let conn = db.get().expect("Database not initialized");

        let mut statement = conn
            .prepare("SELECT sender, receiver, content 
            FROM
            messages 
            WHERE
            created_at > ?1
            AND
            (sender = ?2 OR receiver = ?2)
            ORDER BY created_at")
            .expect("Unable to create SQL statement");
        
        let messages :Vec<Message> =
            statement.query_map((after, npc.name()),
            |row|    
            Ok(Message{
                is_recieved: row.get::<usize,String>(0)? == "Player".to_string(),
                content: row.get::<usize,String>(2)?,
            })
        )?
        .filter(|row| row.is_ok())
        .map(|row| row.unwrap())
        .collect();

        Ok(messages)   
    });

    result.unwrap_or(vec![])

}