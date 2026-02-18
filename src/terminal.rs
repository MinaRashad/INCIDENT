use terminal_size::{Width, Height, terminal_size};
use std::io::{self, Write};
use std::time::Duration;
use crossterm::event::{Event, KeyCode, poll, read};
use strip_ansi_escapes::strip;

/*
    Terminal module
    Here will be only Terminal utilities
    these include:
    1. Changing how the text looks on the screen
    2. Changing low level cursor control
*/

// constants
/// ANSI Control Sequence Introducer - prefix for all ANSI escape codes
pub const CSI :&str="\x1B[";

// terminal utility
// functions that control the terminal pattern

/// Returns the current terminal size as [width, height]
/// Returns [0, 0] if unable to retrieve terminal size
pub fn size() -> [usize;2]
{
    let size: Option<(Width, Height)> = terminal_size();

    if let Some((Width(w), Height(h))) = size {
        [w as usize, h as usize]
    }else{
        println!("Unable to retrieve the terminal size");
        [0,0]
    }
}

/// Clears the entire screen and moves cursor to top-left (1,1)
pub fn clear_screen(){
    print!("{CSI}2J{CSI}1;1H");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Clears the scrollback buffer and moves cursor to top-left (1,1)
pub fn clear_scrollback(){
    print!("{CSI}3J{CSI}1;1H");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Sets the terminal window title
pub fn set_title(title:&str){
    print!("\x1B]0;{title}\x1B\\");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Enables automatic text wrapping at the terminal edge
// pub fn enable_text_warp(){
//     print!("{CSI}?7h");
//     io::stdout().flush()
//         .expect("Failed to flush");

// }

/// Disables automatic text wrapping - text will be truncated at edge
pub fn disable_text_warp(){
    print!("{CSI}?7l");
    io::stdout().flush()
        .expect("Failed to flush");

}

/* Text variations
   each function takes string and outputs a string
   the reason is that I want them to be compatible so that
   I can overlay, XYZ SUMMON
 */
// color functions

/// Applies RGB foreground color to text
/// Returns the text wrapped in ANSI color codes
/// Example: foreground_color("Hello".to_string(), [255, 0, 0]) makes red text
pub fn foreground_color(text:String,color:[u8;3])->String{
    format!("{CSI}38;2;{R};{G};{B}m{text}{CSI}0m", 
        R = color[0],
        G = color[1],
        B = color[2])
}

/// Applies RGB background color to text
/// Returns the text wrapped in ANSI color codes
/// Example: background_color("Hello".to_string(), [0, 0, 255]) makes blue background
pub fn background_color(text:String,color:[u8;3])->String{
    format!("{CSI}48;2;{R};{G};{B}m{text}{CSI}0m", 
        R = color[0],
        G = color[1],
        B = color[2])
}

// format

/// Makes text bold/bright
/// Returns the text wrapped in ANSI bold codes
pub fn bold(text:String) -> String{
    format!("{CSI}1m{text}{CSI}0m")
}

/// Inverts foreground and background colors
/// Returns the text wrapped in ANSI invert codes
pub fn invert(text:String) -> String{
    format!("{CSI}7m{text}{CSI}0m")
}

/// Makes text appear dimmed/faint
/// Returns the text wrapped in ANSI faint codes
pub fn faint(text:String) -> String{
    format!("{CSI}2m{text}{CSI}0m")
}

// animation

/// Makes text blink (if terminal supports it)
/// Returns the text wrapped in ANSI blink codes
pub fn blink(text:String) -> String{
    format!("{CSI}5m{text}{CSI}0m")
}


// positioning

/// Centers a single line of text horizontally on the screen
/// Panics if text is longer than terminal width
/// Uses ANSI cursor positioning to achieve centering
pub fn center(text:String)->String{
    let [w, _] = size();
    let stripped = strip(&text);
    let text_len = String::from_utf8(stripped);
    let text_len = match text_len {
        Ok(s) => s.chars().count(),
        Err(_) => panic!("This should never happen")
    };

    if text_len >= w {
        panic!("The text is too long to be centered :{text}, {w}")
    }

    // to center a text we need the following
    // --space--text len--space-
    // we have 2*space + text len = w
    // space = (w - text len) // 2
    
    let space = (w - text_len) / 2;

    // in ANSI we will set the cursor position to
    // first column then
    
    format!("{}{}", " ".repeat(space), text)
}

/// Centers multiple lines of text, each line centered independently
/// Processes each line through center() and adds newlines between them
pub fn center_multiline(text:String)->String{
    let lines = text.lines();

    let mut result = String::new();

    for line in lines{
        let centered_line = center(line.to_string());
        
        result = result + &centered_line;

        result += "\n";

    }
    result
}


// input
/* Ideal system: */
/* get_input (nonblocking method) -> key pressed (assume one key) */

/// Non-blocking input check - returns immediately
/// Drains all queued inputs first, then waits up to 50ms for new input
/// Returns KeyCode::Null if no input detected
/// Enables raw mode temporarily to read input
pub fn get_input_now() -> KeyCode {
    let mut input = KeyCode::Null;
    crossterm::terminal::enable_raw_mode().
        expect("Failed to enter raw mode");

    // drain all queued inputs
    
    
    if poll(Duration::from_millis(50))
        .is_ok_and(|x| x) 
    {
        // now we polled we can read without blocking
        match read() {
            Ok(Event::Key(event))=>{
                input = event.code;
            },
            Ok(_) =>{}
            Err(_) =>{}
        }


    } 

    
    crossterm::terminal::disable_raw_mode().
        expect("Failed to exit raw mode");

    input
}

/// Blocking input - waits indefinitely for a key press
/// Drains all queued inputs first to get fresh input
/// Returns the KeyCode of the pressed key
pub fn get_input() -> KeyCode{
    let mut input = KeyCode::Null;

    // drain all queued inputs
    drain_input();

    match read() {
                Ok(Event::Key(event))=>{
                    input = event.code;
                },
                Ok(_) =>{}
                Err(_) =>{}
            }

    input
}
pub fn drain_input() {
    while poll(Duration::from_millis(0))
         .is_ok_and(|x| x) 
    {
        let _ = read();
    }
}

// cursor

/// Moves cursor up by specified number of rows
// pub fn move_cursor_up(rows:usize){
//     print!("{CSI}{rows}A");
//     io::stdout().flush()
//         .expect("Failed to flush");
// }

// /// Moves cursor down by specified number of rows
// pub fn move_cursor_down(rows:usize){
//     print!("{CSI}{rows}B");
//     io::stdout().flush()
//         .expect("Failed to flush");
// }

/// Moves cursor right by specified number of columns
pub fn move_cursor_right(cols:usize){
    print!("{CSI}{cols}C");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Moves cursor left by specified number of columns
pub fn move_cursor_left(cols:usize){
    print!("{CSI}{cols}D");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Moves cursor to the beginning of the current line (carriage return)
pub fn move_cursor_linestart(){
    print!("\r");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Hides the terminal cursor
pub fn hide_cursor(){
    print!("{CSI}?25l");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Shows the terminal cursor
pub fn show_cursor(){
    print!("{CSI}?25h");
    io::stdout().flush()
        .expect("Failed to flush");
}


// alternative buffer
// for better screen cleanup

/// Switches to the alternative screen buffer
/// Useful for full-screen applications - preserves the original screen
pub fn enter_alternative_buffer(){
    print!("{CSI}?1049h");
    io::stdout().flush()
        .expect("Failed to flush");
}

/// Exits the alternative screen buffer and returns to main buffer
/// Restores the original screen content
pub fn exit_alternative_buffer(){
    print!("{CSI}?1049l");
    io::stdout().flush()
        .expect("Failed to flush");
}

// scrolling

/// Scrolls the screen down by n lines (content moves up)
// pub fn scroll_down(n:usize){
//     print!("{CSI}{n}S");
//     io::stdout().flush()
//         .expect("Failed to flush");
// }

// /// Scrolls the screen up by n lines (content moves down)
// pub fn scroll_up(n:usize){
//     print!("{CSI}{n}T");
//     io::stdout().flush()
//         .expect("Failed to flush");
// }


// random stuff

/// Converts a string message into ASCII art using FIGlet fonts
/// Uses the standard FIGlet font
/// Returns the original message if conversion fails
pub fn figlet_figure(message:String)-> String{
    let font = figleter::FIGfont::standard()
                            .expect("Unable to retrieve the standard fonts");
    
    let figure = font.convert(&message);

    match figure {
        Some(msg)=>{
           msg.to_string()
        },
        None =>{
            message
        }
    }
}