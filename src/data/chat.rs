use log::error;
use ratatui::widgets::ListState;

use std::{
    collections::{HashMap, HashSet}, 
    fs::File, 
    io::Error, 
    path::PathBuf, 
    thread, 
    time::Duration, 
    time::SystemTime,
    time::UNIX_EPOCH,
    vec
};

use serde::{Deserialize, Serialize};
use serde_json;
use crate::{data, events::EventType, sound};

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
    next_dialogue:Option<String>, // key name of next node 
    state: DialogueNodeStatus
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq)]
enum DialogueNodeStatus{
    #[default]
    NotProcessed,
    WaitingPlayerResponse
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

    // initialization
    let mut chats = read_dialogue_data()
                                .expect("Failed to get a hashmap of the messages");
    let mut timestamp:u32 = 0;

    let mut known_history:HashSet<EventType> = HashSet::new();

    loop {
        // first we check the history to see new events
        let new_history = get_history(timestamp);

        // if there is add it to the hashmap
        for event in &new_history{
            known_history.insert(event.clone());
        }

        // ideally we want to process the dialogues only
        // if we saw new history
        // TODO add a start game event
        // because otherwise the loop will end here at the start 
        // of the game
        if new_history.is_empty(){
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        // now we have a history we should process each
        // npc dialogue
        let npc_node_map = get_dialogue_map();
        for npc in NPC::ALL{
            let node_name = match npc_node_map.get(&npc) {
                    Some(s)=> s,
                     _ => continue};
            

            let dialogue_map = match chats.get_mut(&npc) {
                Some(map)=> map,
                None => continue
            };

            let mut dialogue_node = match dialogue_map.get_mut(node_name) {
                Some(node)=> node,
                None => continue
            };

            if dialogue_node.state == DialogueNodeStatus::WaitingPlayerResponse {
                continue;
            }

            // the dialogue node needs processing 
            // if we arrived here

            // first make sure the conditions are satesfied before we continue
            let mut valid_to_process = true;
            for condition in &dialogue_node.conditions{
                if !known_history.contains(&EventType::from_str(condition)){
                    valid_to_process = false;
                    break;
                }
            }
            if !valid_to_process {continue;}
            
            // now we are graunteed that the node satesfied its conditions
            // and
            process_dialogue_node(&mut dialogue_node, Contact_Char::NPC(npc), Contact_Char::Player);

        }

        // update timestamp
        timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs() as u32;

        thread::sleep(Duration::from_secs(1));
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


fn send_message(from: &Contact_Char, to: &Contact_Char, message:String)
{
    sound::play(sound::SoundCategory::GUIFeedback);
    let result = data::METADATA_DB.with(|db|{
        let conn = db.get().expect("Database not initialized");

        conn.execute(
            "INSERT INTO messages (sender, receiver, content) VALUES (?1, ?2, ?3)", 
            (from.to_string(), to.to_string(), message))
    });

    if result.is_err(){
        error!("Failed to send message")
    }
}

pub fn get_messages(after: u32, from:NPC) 
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
            statement.query_map((after, from.name()),
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

fn get_history(after: u32)
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

pub fn set_dialogue_state(npc: NPC, node: String) {
    let result = data::METADATA_DB.with(|db| {
        let conn = db.get().expect("Database not initialized");

        conn.execute(
            "INSERT INTO npc_dialogue_state (npc_name, node) VALUES (?1, ?2)
             ON CONFLICT(npc_name) DO UPDATE SET node = excluded.node",
            (npc.name(), node))
    });

    if result.is_err() {
        error!("Failed to set dialogue state")
    }
}

pub fn get_dialogue_state(npc: NPC) -> String {
    let result = data::METADATA_DB.with(
        |db| 
        -> Result<String, rusqlite::Error> {
            let conn = db.get().expect("Database not initialized");

            let mut statement = conn
                .prepare("SELECT node FROM npc_dialogue_state WHERE npc_name = ?1")
                .expect("Unable to create SQL statement");

            statement
                .query_row((npc.name(),)
                ,|row| row.get::<usize, String>(0))
        });

    result.unwrap_or("start".to_string())
}


fn get_dialogue_map() -> HashMap<NPC, String>{
    let mut map: HashMap<NPC, String> = HashMap::new();
    for npc in NPC::ALL {
        let curr_node = get_dialogue_state(npc);

        // if its start, save it just in case
        if curr_node == "start".to_string(){
            set_dialogue_state(npc, curr_node.clone());
        }

        map.insert(npc, curr_node);
    }

    map
}


/// # `process_dialogue_node`
/// - Arguments
///     - node: mutable reference to the dialogue node
///     - from: Contact_Char (either player or NPC)
///     - to: Contact_Char (either player or NPC)
/// 
/// Ideally this function can also be used to indicate a choice has been made
pub fn process_dialogue_node(node: &mut DialogueNode, from: Contact_Char, to:Contact_Char){

    // first we send a message with the content of the node
    match &node.text {
        Some(text) => send_message(&from, &to, text.to_string()),
        None => {}
    };

    // then add its events to the history
    for event in &node.events{
        add_event(event.to_string());
    }

    // lets find which contact is the NPC
    let npc = if let Contact_Char::NPC(npc) = to {
        npc
    }else if let Contact_Char::NPC(npc) = from {
        npc
    } else{
        // this should not happen because it would
        // mean there are more than one player
        NPC::Marcus
    };

    // finally there are two cases
    if let Some(next_node) = &node.next_dialogue {
        set_dialogue_state(npc, next_node.to_string());
    }
    // if we dont have a next node, set the node to be waiting for choices
    else {
        node.state = DialogueNodeStatus::WaitingPlayerResponse;
    }

    

}