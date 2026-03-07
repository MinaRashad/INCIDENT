
// marking a contradiction will take the following steps

use std::path::PathBuf;

use crate::{data::docs, game_state::GameState, menu_components, terminal, views::docs::{DOCS_ROOT, choose_file}};

pub fn mark_contradiction(first_doc:PathBuf) -> GameState {

    // first we need them to select the second document
    // terminal::clear_screen();
    // terminal::clear_scrollback();
    // println!("Under Construction");
    // menu_components::wait_for_input();
    // return GameState::MainMenu;

    let second_document = choose_file("Where is the contradiction");

    let Some(second_document) = second_document
    else {return GameState::OpenPath(first_doc);};

    if second_document == first_doc {
        print_error("The document does not contradict itself");
        return GameState::OpenPath(first_doc);
    }


    GameState::OpenPath(first_doc)
}


fn print_error(text:&str){
    terminal::clear_screen();
    terminal::clear_scrollback();
    
    let title = terminal::figlet_figure("ERROR".to_string());
    let title = terminal::center_multiline(title);
    let title = terminal::foreground_color(title, [255, 50, 50]);
    
    println!("{}", title);
    println!();
    println!();
    
    let error_text = terminal::center_multiline(text.to_string());
    let error_text = terminal::bold(error_text);
    
    println!("{}", error_text);
    println!();
    
    menu_components::wait_for_input();
}