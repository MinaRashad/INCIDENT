mod terminal;
mod menu_components;
mod views;
mod animate;
mod windows;
mod data;
mod sound;
mod game_state;
mod util;
mod events;

use std::{env, io::Error};
use log;
use env_logger;
use game_state::GameState;

use crate::data::chat;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::exit_alternative_buffer();
    }
}




fn main() {
    let _guard = CleanUp;
    let mut state = GameState::TitleScreen;

    let args :Vec<String>= env::args().collect();
    println!("{args:?}");

    let _ = init();


    if args.len() > 1 {
        state = match &args[1] {
            p if p == "--docs" => GameState::Docs,
            p if p == "--chats" => GameState::Chats,
            _ => GameState::MainConsole
        }
    }
    
    loop {
        state = state.run();
    };

}




fn init()->Result<(), Error>{
    // terminal::enter_alternative_buffer();
    terminal::set_title("INCIDENT");
    sound::init()?;
    data::init_db();
    data::player::init_player();
    events::init_events();
    env_logger::init();

    // TEMPORARY: spawn the chat
    chat::spawn_chat_master();
    
    Ok(())
}