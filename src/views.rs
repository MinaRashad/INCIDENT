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

pub fn main_menu() -> GameState {
    terminal::clear_screen();
    terminal::hide_cursor();

    // GRUB uses blue background
    let blue_bg = [0, 0, 128];
    let white_fg = [255, 255, 255];
    let light_gray = [200, 200, 200];

    // Create full-width colored header
    let [width, _] = terminal::size();
    let padding = " ".repeat(width);
    
    // GRUB header with background
    let header_line = terminal::background_color(
        terminal::foreground_color(padding.clone(), white_fg),
        blue_bg
    );
    
    let title_text = format!("{:^width$}", "GNU GRUB  version 2.06", width = width);
    let grub_title = terminal::background_color(
        terminal::foreground_color(title_text, white_fg),
        blue_bg
    );

    println!("{}", header_line);
    println!("{}", grub_title);
    println!("{}", header_line);
    println!();

    // Get the selection
    let choice = menu_components::multichoice(
        "",
        vec![
            GameState::Startup,
            GameState::Options,
            GameState::Exit
        ],
        true
    );

    println!();
    println!();
    

    terminal::show_cursor();
    choice
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
