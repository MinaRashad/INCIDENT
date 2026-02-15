use crate::data;
use crate::data::docs::metadata;
use crate::data::sha256;
use crate::terminal;
use crate::animate;
use crate::menu_components;
use crate::GameState;

pub mod chat;
pub mod docs;
pub mod game;

use std::path::PathBuf;
use data::{Entry};

// main menu
// just the title screen, and input block

/// Creates the game title "INCIDENT" as styled ASCII art
/// 
/// # Returns
/// String containing the title in FIGlet font, centered, bold, and green colored
fn title()->String{
    let title = "INCIDENT".to_string();

    let title = terminal::figlet_figure(title);

    let title = terminal::center_multiline(title);

    let title = terminal::bold(title);

    

    terminal::foreground_color(title, [100,250,100])
}

/// Displays the title screen and waits for user input
/// 
/// # Returns
/// GameState::MainMenu after user presses a key
/// 
/// Clears screen, shows the title, waits for input, then transitions to main menu
pub fn title_page()->GameState{
    terminal::clear_screen();

    println!("{}",title());

    menu_components::wait_for_input();
    terminal::clear_screen();

    GameState::MainMenu
}

/// Displays a GRUB-style boot menu with game options
/// 
/// # Returns
/// The selected GameState (Startup, Options, or Exit)
/// 
/// Styled to look like GNU GRUB bootloader:
/// - Blue background header (RGB: 0, 0, 128)
/// - White text on blue background
/// - Centered menu options
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


/// Displays a welcome greeting with username and current date
/// 
/// # Arguments
/// * `color1` - Text color for the username greeting
/// * `bgcolor1` - Background color for the username greeting
/// * `color2` - Text color for the date
/// * `bgcolor2` - Background color for the date
/// 
/// Shows:
/// - "Welcome back, [username]" (currently hardcoded to "admin")
/// - Current system date in YYYY-MM-DD format
/// Both lines are centered and colored according to the provided RGB values
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

/// Displays an "ACCESS DENIED" screen for unauthorized file access attempts
/// 
/// # Arguments
/// * `path` - PathBuf of the file that was denied access
/// 
/// # Returns
/// GameState::GoBack with the parent directory path
/// 
/// Shows:
/// 1. Flashing "ACCESS DENIED" message in red (3 times, 150ms intervals)
/// 2. Restricted filename warning in yellow
/// 3. "INSUFFICIENT CLEARANCE LEVEL" prompt in inverted red
/// 4. Waits for user to press any key
/// 
/// After keypress, returns to the parent directory
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

/// Displays a password prompt for password-protected files
/// 
/// # Arguments
/// * `path` - PathBuf of the password-protected file
/// 
/// # Returns
/// - GameState::OpenPath(path) if password is correct
/// - GameState::Unauthorized(path) if password is incorrect
/// 
/// # Panics
/// Panics if the file has no password metadata (shouldn't happen if called correctly)
/// 
/// Shows:
/// - "LOCKED" FIGlet text in yellow
/// - Password input prompt
/// Compares SHA-256 hash of input against stored hash
pub fn password_access(path: PathBuf) -> GameState {
    let entry = Entry { path: path.clone() };
    
    if let Some(data) = metadata(&entry) &&
       let Some(password_sha256hash) = data.password {
        
        terminal::clear_screen();
        println!("\n");
        
        let lock = terminal::figlet_figure("LOCKED".to_string());
        let yellow = [255, 200, 0];
        print!("{}", terminal::center_multiline(
            terminal::foreground_color(lock, yellow)
        ));
        
        println!("\n{}", terminal::center(
            terminal::bold("Enter password:".to_string())
        ));
        print!("{}", terminal::center("→ ".to_string()));
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
        
        let hashed = data::sha256(input);
        
        if hashed == password_sha256hash {
            GameState::OpenPath(path)
        } else {
            GameState::Unauthorized(path)
        }
    } else {
        panic!("SHOULD NEVER HAPPEN")
    }
}