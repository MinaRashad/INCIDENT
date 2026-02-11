
use crate::menu_components;
use crate::GameState;
use crate::data::player;


pub fn start()->GameState{    
    let user = player::get_player();
    println!("Hello {user:?}");


    menu_components::wait_for_input();

    GameState::Chats
}
