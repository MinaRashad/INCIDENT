use crate::terminal;
use crate::menu_components;
use crate::GameState;

pub fn start()->GameState{
    println!("Chat is working");
    menu_components::wait_for_input();

    GameState::Chats
}