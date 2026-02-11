mod terminal;
mod menu_components;
mod views;
mod animate;
mod windows;
mod data;
mod sound;


use std::{env, io::Error};


struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::exit_alternative_buffer();
    }
}

enum GameState{
    TitleScreen,
    MainMenu,
    Options,
    // main game loop
    Startup,
    MainConsole,
    Chats,
    Docs,
    Exit
}

impl GameState {
    fn run(self)->GameState{
        match self {
                    GameState::TitleScreen => views::title_page(),
                    GameState::MainMenu => views::main_menu(),
                    GameState::Startup => views::game::start_up(),
                    GameState::MainConsole => views::game::main_console(),
                    GameState::Options => todo!("No options yet"),
                    GameState::Chats => views::chat::start(),
                    GameState::Docs => views::docs::start(),
                    GameState::Exit => std::process::exit(0),
                }
    }
}

fn main() {
    let _guard = CleanUp;
    let mut game_state = GameState::TitleScreen;

    let args :Vec<String>= env::args().collect();
    println!("{args:?}");

    let _ = init();


    if args.len() > 1 {
        if args[1] == "--chat".to_string(){
            game_state = GameState::Chats
        }else if args[1] == "--docs".to_string() {
            game_state = GameState::Docs
        }
    }
    
    loop {
        game_state = game_state.run();
    };

}




fn init()->Result<(), Error>{
    terminal::enter_alternative_buffer();
    sound::init()?;
    data::player::init_player();

    Ok(())
}