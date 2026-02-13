mod terminal;
mod menu_components;
mod views;
mod animate;
mod windows;
mod data;
mod sound;


use std::{env, io::Error, path:: PathBuf};


struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::exit_alternative_buffer();
    }
}
#[derive(Clone)]
enum GameState{
    TitleScreen,
    MainMenu,
    Options,
    // main game loop
    Startup,
    MainConsole,
    Chats,
    Docs,
    OpenPath(PathBuf),
    GoBack(PathBuf),
    NewWindow(String),
    Exit
}

impl GameState {
    fn as_name(&self) -> String{
        match &self {
            GameState::Chats => "Chat Log".to_string(),
            GameState::Docs => "Documents".to_string(),
            GameState::MainConsole => "Main Console".to_string(),
            GameState::OpenPath(path) => path.file_name()
                                        .and_then(|f| f.to_str())
                                        .and_then(|s| Some(s.to_string()))
                                        .unwrap_or("UNKNOWN FILE".to_string())
                                    ,
            GameState::GoBack(path) => "Back".to_string(),
            GameState::Exit => "Exit".to_string(),
            GameState::Options => "Options".to_string(),
            GameState::Startup => "Boot".to_string(),
            GameState::TitleScreen => "Title screen".to_string(),
            GameState::MainMenu => "Main Menu".to_string(),
            GameState::NewWindow(name) => format!("Open {name}", name=name.to_uppercase())
        }
    }
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
                    GameState::OpenPath(path) => views::docs::open_path(path),
                    GameState::GoBack(path) => views::docs::open_path(path),
                    GameState::NewWindow(name) => {
                        windows::start_mode(name.as_str());
                        GameState::MainConsole
                    }
                }
    }
}

fn main() {
    let _guard = CleanUp;
    let mut game_state = GameState::MainConsole;// GameState::TitleScreen;

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
    terminal::set_title("INCIDENT");
    sound::init()?;
    data::player::init_player();
    Ok(())
}