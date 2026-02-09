use crate::terminal;
use crate::menu_components;
use crate::GameState;

pub fn start()->GameState{
    println!("Docs is working");
    menu_components::wait_for_input();

    return GameState::Docs;
}