use crate::terminal;
use crate::menu_components;
use crate::GameState;

pub mod chat;
pub mod docs;
pub mod game;



// main menu
// just the title screen, and input block
fn title()->String{
    let title = "INCIDENT".to_string();

    let title = terminal::figlet_figure(title);

    let title = terminal::center_multiline(title);

    let title = terminal::bold(title);

    

    terminal::foreground_color(title, [100,250,100])
}

pub fn title_page()->GameState{
    terminal::clear_screen();

    println!("{}",title());

    menu_components::wait_for_input();
    terminal::clear_screen();

    GameState::MainMenu
}

pub fn main_menu()->GameState{
    terminal::clear_screen();

    println!("{}",title());


    let selection = menu_components::multichoice(
        "Main Menu", 
        vec!["Start", "Options", "Exit"],
        true
        );
    
    if selection == 0{
        return GameState::Startup;
    }else if selection == 1 {
        return GameState::Options;
    } else{
        return GameState::Exit;
    }
   
}




pub fn print_greeting(color1:[u8;3], bgcolor1:[u8;3], 
                                 color2:[u8;3], bgcolor2:[u8;3])
{
    terminal::clear_screen();
    let user_name = "admin";
    let greeting = "Welcome back, ".to_string() + user_name;

    let greeting = terminal::center(greeting);
    let greeting = terminal::foreground_color(greeting.to_string(), color1);
    let greeting = terminal::background_color(greeting, bgcolor1);

    println!("{greeting}");

    let date = menu_components::date()
                            .expect("getting a valid date");
    let date = format!("{0}-{1}-{2}",date[0], date[1],date[2]);
    let date = terminal::center(date);
    let date = terminal::foreground_color(date, color2);
    let date = terminal::background_color(date, bgcolor2);

    println!("{date}");
}
