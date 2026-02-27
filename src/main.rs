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

use crate::{data::chat, game_state::endings::{self, Ending}};

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

    // check if the game ended then switch to that ending
    if let Some(ending) = endings::get_ending(){
        state = GameState::Ending(ending)
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
    env_logger::init();

    // TEMPORARY: spawn the chat
    chat::spawn_chat_master();
    events::spawn_all_seeing_eye();
    
    Ok(())
}