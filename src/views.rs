
use crate::terminal;
use crate::animate;
use crate::menu_components;
use crate::GameState;

pub mod chat;
pub mod docs;
pub mod game;

use std::path::PathBuf;

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


pub fn unauthorized_access(path: PathBuf) -> GameState {
    terminal::clear_screen();
    terminal::hide_cursor();
    
    let previous_path = path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(docs::DOCS_ROOT));
    
    let warning = terminal::figlet_figure("ACCESS".to_string());
    let denied = terminal::figlet_figure("DENIED".to_string());
    
    let red = [255, 50, 50];
    let dark_red = [150, 0, 0];
    
    // Create flash content
    let colored_warning = terminal::foreground_color(
        terminal::bold(warning.clone()),
        red
    );
    let colored_denied = terminal::foreground_color(
        terminal::bold(denied.clone()),
        red
    );
    
    let flash_content = format!("\n\n{}{}", 
        terminal::center_multiline(colored_warning),
        terminal::center_multiline(colored_denied)
    );
    
    // Flash effect
    animate::flash(flash_content, 150, 3);
    
    // Final display
    terminal::clear_screen();
    let colored_warning = terminal::foreground_color(
        terminal::bold(warning),
        red
    );
    let colored_denied = terminal::foreground_color(
        terminal::bold(denied),
        dark_red
    );
    
    print!("{}", terminal::center_multiline(colored_warning));
    print!("{}", terminal::center_multiline(colored_denied));
    
    println!();
    println!();
    
    let filename = path.file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("UNKNOWN");
    
    let error_msg = format!("⚠ RESTRICTED FILE: {}", filename);
    let error_colored = terminal::foreground_color(
        terminal::bold(error_msg),
        [255, 200, 0]
    );
    println!("{}", terminal::center(error_colored));
    
    println!();
    let prompt = "[ INSUFFICIENT CLEARANCE LEVEL ]";
    let prompt_colored = terminal::foreground_color(
        terminal::invert(prompt.to_string()),
        red
    );
    println!("{}", terminal::center(prompt_colored));
    
    println!();
    println!();
    
    let instruction = terminal::faint("Press any key to return...".to_string());
    println!("{}", terminal::center(instruction));
    
    terminal::get_input();
    terminal::show_cursor();
    
    GameState::GoBack(previous_path)
}