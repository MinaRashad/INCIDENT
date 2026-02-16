use std::io::{self, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{self};
use crossterm::event::KeyCode;
use rascii_art;

use crate::data::{self, ImageDoc};
use crate::terminal;
use crate::sound;
use crate::game_state::GameState;



// complex menus

/// Displays a list of options on screen
/// 
/// # Arguments
/// * `options` - Vector of option strings to display
/// * `centered` - If true, centers each option horizontally on screen
/// 
/// # Returns
/// Returns the same vector of options (for chaining or further processing)
/// 
/// Prints each option on a new line and flushes output to ensure immediate display
fn display_options(options:Vec<String>, centered:bool) -> Vec<String>{
    for i in 0..options.len(){
        let option = &options[i];
        let option = option.to_string();
        let option = if centered{terminal::center(option)} 
                     else {option};
        println!("{option}");
    }

    // flush everything
    io::stdout().flush()
            .expect("Failed to flush");

    options
}

/// Highlights a menu option by inverting its colors
/// Moves cursor to the option's position, applies invert style, then returns cursor
/// 
/// # Arguments
/// * `option` - The option text to highlight
/// * `num_options` - Total number of options in the menu
/// * `curr_selection` - Index of the option to highlight (0-based)
/// * `centered` - Whether the option should be centered
fn highlight_option(option:String, num_options:usize,
                    curr_selection:usize,
                    centered:bool){
        // move cursor up option.len - curr_selection
        terminal::move_cursor_up(num_options - curr_selection);
        terminal::move_cursor_linestart();

        //center first
        let option = if centered{terminal::center(option)} 
                             else {option};

        // first we need to print the inverted text at the current selection
        let selection = terminal::invert(option);
        print!("{}",selection);

        // flush the output:
        io::stdout().flush()
            .expect("Failed to flush");
        
        // move cursor down curr_selection 
        terminal::move_cursor_down( num_options - curr_selection);
        
}

/// Removes highlight from a menu option by printing it normally
/// Moves cursor to the option's position, prints normal text, then returns cursor
/// 
/// # Arguments
/// * `option` - The option text to unhighlight
/// * `num_options` - Total number of options in the menu
/// * `curr_selection` - Index of the option to unhighlight (0-based)
/// * `centered` - Whether the option should be centered
fn unhighlight_option(option:String, 
                        num_options:usize, 
                        curr_selection:usize,
                        centered:bool){
        // move cursor up option.len - curr_selection
        terminal::move_cursor_up(num_options - curr_selection);
        terminal::move_cursor_linestart();

        //center first
        let option = if centered{terminal::center(option)} 
                             else {option};
        
        print!("{}",&option);

        // flush the output:
        io::stdout().flush()
            .expect("Failed to flush");
        
        // move cursor down curr_selection 
        terminal::move_cursor_down( num_options - curr_selection);
        
}

/// Displays an interactive menu with keyboard navigation
/// User can navigate with arrow keys (↕) and select with Enter (↩)
/// 
/// # Arguments
/// * `title` - Title text displayed above the menu
/// * `options` - Vector of GameState options to choose from
/// * `centered` - If true, centers all text horizontally
/// 
/// # Returns
/// The selected GameState
/// 
/// # Panics
/// Panics if options vector is empty
/// 
/// Controls:
/// - Up/Down arrows: Navigate through options
/// - Enter: Confirm selection
/// Includes input debouncing (200ms) and audio feedback on navigation
pub fn multichoice(title:&str, options:Vec<GameState>,
                    centered:bool)-> GameState{

    
    terminal::hide_cursor();
    // handle unexpected cases
    if options.is_empty() {panic!("There are no options")};

    let options_str: Vec<String> = options.iter()
                .map(|state| state.as_name())
                .collect();

    // print the title
    let title = if centered {terminal::center(title.to_string())} 
                        else{title.to_string()};
    let title = terminal::bold(title);
    let help = "Select your choice (Use ↕ and ↩)".to_string();
    let help = if centered{ terminal::center(help)}
                       else{help};
    let help = terminal::blink(help);
    
    println!("{}",title);
    println!("{}", help);

    let options_str =
        options_str.iter().map(
                |s| s.to_string()
            )
            .collect();

    let options_str = display_options(options_str, centered);
    

    let mut curr_selection :usize= 0;

    // input buffer
    let input_buffer = time::Duration::from_millis(200);
    let mut now = time::SystemTime::now();

    loop{
        highlight_option(options_str[curr_selection].to_string(),
                        options_str.len(),
                        curr_selection,
                        centered);

        let elapsed = now.elapsed().expect("Getting elapsed time failed");
        if elapsed < input_buffer {
            sleep(input_buffer - elapsed);
        }

        // now we can wait for an input
        let input = terminal::get_input();
        
        if input.is_down() || input.is_up(){

            unhighlight_option(options_str[curr_selection].to_string(),
                        options_str.len(),
                        curr_selection,
                        centered);

            now = time::SystemTime::now();

            sound::play(sound::SoundCategory::GUIFeedback);

        }
        if input.is_down() {
            curr_selection = (curr_selection + 1) % options.len();
        } else if input.is_up() {
            curr_selection =  if curr_selection == 0 {curr_selection + options.len()} else {curr_selection};
            curr_selection -= 1;
        } else if input.is_enter() {
            break;
        }

    }

    options[curr_selection].clone()
}


// graphical functions

/// Renders an image as colored ASCII art
/// 
/// # Arguments
/// * `img` - ImageDoc containing the path to the image file
/// * `w` - Optional width constraint in characters
/// * `h` - Optional height constraint in characters
/// 
/// # Returns
/// Some(String) containing the ASCII art if successful, None if rendering fails
/// 
/// Uses the rascii_art library to convert images to colored block characters
/// Respects width/height constraints if provided
pub fn display_image(img:ImageDoc, w:Option<u32>, h:Option<u32>)-> Option<String>{

    let mut image_ascii = String::new();
    let options = rascii_art::RenderOptions::new();
    let options = options.colored(true);
    let mut options = options.charset(rascii_art::charsets::BLOCK);

    if let Some(w) = w{
        options = options.width(w);
    }
    
    if let Some(h) = h {
        options = options.height(h);
    }

    let path = img.get_path();

    let path = path.to_str()?;

    let result = rascii_art::render_to(path, &mut image_ascii, 
        &options
        );
    
    if result.is_ok(){
        return Some(image_ascii);
    }

    return None
}

/// Displays the OS logo as ASCII art
/// Logo is scaled to 70% of terminal width and centered on screen
/// Uses the OS_LOGO_PATH from the data::docs module
pub fn print_logo(){
    let [w, h] = terminal::size();
    let w = w as u32;
    let h= h as u32;

    let w = (70*w)/100;
    let logo_path = data::docs::OS_LOGO_PATH;
    let logo_path = PathBuf::from(logo_path);
    let logo = ImageDoc::image(logo_path);
    
    if let Some(img) = display_image(logo, Some(w), None)
    {
        //let img = terminal::center_multiline(img);
        terminal::move_cursor_linestart();
        let img = terminal::center_multiline(img);
        println!("{}",img)
    }
}

/// Calculates the current date as [year, month, day]
/// 
/// # Returns
/// Ok([year, month, day]) if successful
/// Err if system time is before UNIX epoch
/// 
/// Computes the date from UNIX epoch (1970-01-01) forward
/// Accounts for leap years following Gregorian calendar rules:
/// - Divisible by 4: leap year
/// - Divisible by 100: not a leap year
/// - Divisible by 400: leap year
pub fn date()->Result<[u64; 3],time::SystemTimeError>
{
    

    // let date = format!("{day}-{month}-2230",day=day-sum, month=month);
    let now = time::SystemTime::now();
    // epoch is 1-1-1970
    let interval = now.duration_since(time::UNIX_EPOCH)?;

    let interval = interval.as_secs();

    // now we have how many seconds passed
    // a year on average has 356 day (unless divisible by 4)
    let minutes = interval/60;
    let hours = minutes/60;
    let mut days = hours/24;

    let mut curr_year = 1970;
    let month_days = [31,28,31,30,31,30,31,31,30,31,30,31];
    
    // finding current year
    while days > 365{
        // account for leap day
        if curr_year % 4 == 0{
            if curr_year % 100 == 0 && curr_year % 400 == 0{
                days -= 1 // leap day
            }
            else if curr_year % 100 == 0 && curr_year % 400 != 0{
                // nothing happens
            }
            else{
                days -= 1
            }
        }

        for month in month_days{
            days -= month as u64;
        }
        curr_year += 1
    }
    let mut curr_month  = 0;

    for month in month_days{
        curr_month += 1;

        days -= month as u64;
        if days <= 31{
            break
        }
    }

     

    Ok([curr_year,curr_month,days])
}

/// Displays a "Press anything to Continue..." prompt and waits for input
/// Text is centered and blinking
/// Waits for user to press Enter before continuing
pub fn wait_for_input(){
    let sub = "Press anything to Continue...".to_string();
    let sub = terminal::center(sub);
    let sub = terminal::blink(sub);
    println!("{}",sub);

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("An error occured");
}

/// Displays scroll instructions and waits for Enter key
/// Shows "Use Mouse wheel to scroll. Enter to exit" prompt
/// Text is centered and blinking
/// Loops until user presses Enter
pub fn wait_for_scroll(){
    let sub = "Use Mouse wheel to scroll. Enter to exit".to_string();
    let sub = terminal::center(sub);
    let sub = terminal::blink(sub);
    println!("{}",sub);


    loop {
        let input: KeyCode = terminal::get_input();

        match input {
            KeyCode::Enter => break,
            _ => {}
        }
    }
}