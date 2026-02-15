use std::path::PathBuf;
use crate::views;
use crate::windows;
use crate::terminal;

#[derive(Clone)]
pub enum GameState{
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
    Exit,

    // information control states
    Unauthorized(PathBuf), // Unauthorized access
}

impl GameState {
    pub fn as_name(&self) -> String{
        match &self {
            GameState::Chats => "Chat Log".to_string(),
            GameState::Docs => "Documents".to_string(),
            GameState::MainConsole => "Main Console".to_string(),
            GameState::OpenPath(path) => path.file_name()
                                        .and_then(|f| f.to_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or("UNKNOWN FILE".to_string())
                                    ,
            GameState::GoBack(_) => "Back".to_string(),
            GameState::Exit => "Exit".to_string(),
            GameState::Options => "Options".to_string(),
            GameState::Startup => "Boot".to_string(),
            GameState::TitleScreen => "Title screen".to_string(),
            GameState::MainMenu => "Main Menu".to_string(),
            GameState::NewWindow(name) => format!("Open {name}", name=name.to_uppercase()),

            GameState::Unauthorized(path)=> terminal::faint(
                                GameState::OpenPath(path.to_path_buf())
                                                                .as_name())
        }
    }
}

impl GameState {
    pub fn run(self)->GameState{
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
                    },
                    GameState::Unauthorized(path)=> views::unauthorized_access(path)
                }
    }
}


