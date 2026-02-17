use std::path::PathBuf;
use ratatui::style::Style;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ansi_to_tui::IntoText;

use crate::views;
use crate::windows;
use crate::terminal;

/// Represents all possible states/screens in the game
/// Each state corresponds to a different view or action
/// States can carry data (like file paths) needed for that view
#[derive(Clone)]
pub enum GameState{
    /// Initial title/splash screen
    TitleScreen,
    /// Main menu selection screen
    MainMenu,
    /// Settings/configuration screen
    Options,
    
    // main game loop
    /// Game startup/boot sequence animation
    Startup,
    /// Primary game console/terminal interface
    MainConsole,
    /// Chat log viewer
    Chats,
    /// Document browser/file explorer
    Docs,
    /// Open and display a specific file at the given path
    OpenPath(PathBuf),
    /// Navigate back to parent directory of the given path
    GoBack(PathBuf),
    /// Launch a new window/application with the given name
    NewWindow(String),
    /// Exit the game
    Exit,

    // information control states
    /// Display unauthorized access screen for the given path
    Unauthorized(PathBuf),
    /// Prompt for password before accessing the given path
    PasswordProtected(PathBuf)
}

impl GameState {
    /// Returns a human-readable display name for the current state
    /// Used for UI elements like breadcrumbs or status displays
    /// Some states like Unauthorized use special formatting (faint text)
    /// PasswordProtected states show an asterisk (*) suffix
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
                                                                .as_name()),
            GameState::PasswordProtected(path) => format!("{}*", 
                                            GameState::OpenPath(path.to_path_buf())
                                                                .as_name())
        }
    }
    pub fn as_listitem(&self) -> ListItem<'static> {
        ListItem::new(self.as_name()
                            .into_text()
                            .unwrap_or_default())
    }
}

impl GameState {
    /// Executes the current game state and returns the next state
    /// This is the main state machine driver - each state determines what comes next
    /// Some states exit the program, others transition to new views
    /// The returned GameState is then run in the main game loop
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
                    GameState::Unauthorized(path)=> views::unauthorized_access(path),
                    GameState::PasswordProtected(path) => views::password_access(path)
                }
    }
}