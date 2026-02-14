mod terminal;
mod menu_components;
mod views;
mod animate;
mod windows;
mod data;
mod sound;
mod game_state;

use std::{env, io::Error};
use game_state::GameState;

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
        if args[1] == "--docs" {
            state = GameState::Docs
        }
    }
    
    loop {
        state = state.run();
    };

}




fn init()->Result<(), Error>{
    terminal::enter_alternative_buffer();
    terminal::set_title("INCIDENT");
    sound::init()?;
    data::player::init_player();
    Ok(())
}