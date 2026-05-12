mod terminal;
mod menu_components;
mod views;
mod animate;
mod windows;
mod data;
mod game_state;
mod util;
mod events;

// `sound` has two implementations behind one API: the native one (rodio) and a
// wasm one that forwards to the browser's Web Audio via `wasm_host`.
#[cfg(not(target_arch = "wasm32"))]
mod sound;
#[cfg(target_arch = "wasm32")]
#[path = "sound_wasm.rs"]
mod sound;

// Imports the browser host exposes to the wasm build (sound, "open window",
// terminal size). Not compiled on native.
#[cfg(target_arch = "wasm32")]
mod wasm_host;

use std::{env, io::Error, path::PathBuf};
use log;
use env_logger;
use game_state::GameState;

// some random stuff used for testing
use crate::{data::chat,
  game_state::endings::{self, Ending},
  views::docs::DOCS_ROOT};

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
        // On wasm there are no threads, so the chat/event "masters" can't run
        // in the background — pump them once per state transition instead.
        #[cfg(target_arch = "wasm32")]
        {
            chat::tick();
            events::tick();
        }
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

    // Background processors. On native these are real threads; on wasm they're
    // pumped cooperatively from the game loop (and from the chat view's poll
    // loop) via `chat::tick()` / `events::tick()`.
    #[cfg(not(target_arch = "wasm32"))]
    {
        chat::spawn_chat_master();
        events::spawn_all_seeing_eye();
    }

    Ok(())
}