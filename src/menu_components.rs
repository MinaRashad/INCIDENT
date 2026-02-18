use std::io;
use std::path::PathBuf;
use std::time::{self, Duration};

use crossterm::event::read;
use rascii_art;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode},
};



use crate::data::{self, ImageDoc};
use crate::terminal;
use crate::sound;
use crate::game_state::GameState;



// complex components



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




pub fn multichoice( title: &str, options: Vec<GameState>, centered: bool) -> GameState {
    if options.is_empty() {
        panic!("There are no options");
    }

    if enable_raw_mode().is_err() {
        panic!("Failed to enable raw mode");
    }
    
    std::thread::sleep(Duration::from_millis(100));
    crate::terminal::drain_input();

    let title = title.replace("\\", "/");
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = match ratatui::Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => panic!("Failed to create terminal"),
    };

    let mut selection_state = ListState::default();
    selection_state.select(Some(0));



    loop {
        // Rendering
        terminal.draw(|f| 
            multichoice_render(f, title.clone(),
             &options, 
             centered,
             &mut selection_state
                    ) 
        )
        .expect("Unable to draw the frame");

        //input handling
        let key = match event::read() {
            Ok(Event::Key(k)) => k,
            _ => continue,
        };

        if ! key.is_release(){
            continue;
        }

        if key.code == KeyCode::Enter  {
            break;
        }
        

        let current = match selection_state.selected() {
            Some(i) => i,
            None => continue,
        };

        let next = if key.code == KeyCode::Down {
            (current + 1) % options.len()
        } else if current == 0 {
            options.len() - 1
        } else {
            current - 1
        };

        selection_state.select(Some(next));
        sound::play(sound::SoundCategory::GUIFeedback);
    }

    let _ = disable_raw_mode();

    options[selection_state.selected().unwrap_or(0)].clone()
}

fn multichoice_render(frame: &mut Frame, title:String, 
    options: &Vec<GameState>, centered: bool, 
    selection_state:&mut ListState){
    // This finds the exact size of the terminal at each frame
    // This is much better than the previous approach
    let size = frame.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(size);

    let title_widget = Paragraph::new(title.as_str())
        .alignment(if centered { Alignment::Center } else { Alignment::Left })
        .style(Style::default().add_modifier(Modifier::BOLD));

    let help_widget = Paragraph::new("Select your choice (Use ↑/↓ and Enter)")
        .alignment(if centered { Alignment::Center } else { Alignment::Left });

    let items: Vec<ListItem> = options
        .iter()
        .map(|o| o.as_listitem())
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_widget(title_widget, layout[0]);
    frame.render_widget(help_widget, layout[1]);
    frame.render_stateful_widget(list, layout[2], &mut selection_state.clone());
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
pub fn wait_for_input() -> Option<()> {
    let sub = "Press anything to Continue...".to_string();
    let sub = terminal::center(sub);
    let sub = terminal::blink(sub);
    println!("{}",sub);

    loop{
        if let Event::Key(k) = read().ok()?{
            if k.is_release(){
                break Some(());
            }
        }
    }
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


