use terminal_size::{Width, Height, terminal_size};
use std::io::{self, Write};
use std::time::Duration;
use crossterm;
use crossterm::event::{Event, KeyCode, poll, read};

use figleter;

/*
    Terminal module
    Here will be only Terminal utilities
    these include:
    1. Changing how the text looks on the screen
    2. Changing low level cursor control
*/


// terminal utility
pub fn size() -> [usize;2]
{
    let size: Option<(Width, Height)> = terminal_size();

    if let Some((Width(w), Height(h))) = size {
        return [w as usize, h as usize]
    }else{
        println!("Unable to retrieve the terminal size");
        return [0,0];
    }
}

pub fn clear_screen(){
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush()
        .expect("Failed to flush");
}

pub fn set_title(title:&str){
    print!("\x1B]0;{title}\x1B\\");
    io::stdout().flush()
        .expect("Failed to flush");
}



/* Text variations
   each function takes string and outputs a string
   the reason is that I want them to be compatible so that
   I can overlay, XYZ SUMMON
 */
// color functions

pub fn foreground_color(text:String,color:[u8;3])->String{
    format!("\x1B[38;2;{R};{G};{B}m{text}\x1B[0m", 
        R = color[0],
        G = color[1],
        B = color[2])
}

pub fn background_color(text:String,color:[u8;3])->String{
    format!("\x1B[48;2;{R};{G};{B}m{text}\x1B[0m", 
        R = color[0],
        G = color[1],
        B = color[2])
}

// format
pub fn bold(text:String) -> String{
    format!("\x1B[1m{text}\x1B[0m")
}

pub fn invert(text:String) -> String{
    format!("\x1B[7m{text}\x1B[0m")
}

// animation
pub fn blink(text:String) -> String{
    format!("\x1B[5m{text}\x1B[0m")
}


// positioning
pub fn center(text:String)->String{
    let [w, _] = size();
    let text_len = text.len();

    if text_len >= w {
        panic!("The text is too long to be centered")
    }

    // to center a text we need the following
    // --space--text len--space-
    // we have 2*space + text len = w
    // space = (w - text len) // 2
    
    let space = (w - text_len) / 2;

    // in ANSI we will set the cusoror position to
    // first column then
    
    format!("\x1B[{space}C{text}")
}

pub fn center_multiline(text:String)->String{
    let lines = text.lines();

    let mut result = String::new();

    for line in lines{
        let centered_line = center(line.to_string());
        
        result = result + &centered_line;

        result = result + "\n";

    }
    return result;
}


// input
/* Ideal system: */
/* get_input (nonblocking method) -> key pressed (assume one key) */
pub fn get_input_now() -> KeyCode {
    let mut input = KeyCode::Null;
    crossterm::terminal::enable_raw_mode().
        expect("Failed to enter raw mode");

    // drain all queued inputs
    while poll(Duration::from_millis(0)).unwrap() {
        let _ = read();
    }
    
    match poll(Duration::from_millis(50)) {
        Ok(_)=> {
            // now we polled we can read without blocking
            match read() {
                Ok(Event::Key(event))=>{
                    input = event.code;
                },
                Ok(_) =>{}
                Err(_) =>{}
            }


        },
        Err(_)=> {}
    } 

    
    crossterm::terminal::disable_raw_mode().
        expect("Failed to exit raw mode");

    return input;
}

// blocking version 
pub fn get_input() -> KeyCode{
    let mut input = KeyCode::Null;

    // drain all queued inputs
    while poll(Duration::from_millis(0)).unwrap() {
        let _ = read();
    }

    match read() {
                Ok(Event::Key(event))=>{
                    input = event.code;
                },
                Ok(_) =>{}
                Err(_) =>{}
            }

    return input
}

// cursor
pub fn move_cursor_up(rows:usize){
    print!("\x1B[{rows}A");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_down(rows:usize){
    print!("\x1B[{rows}B");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_right(cols:usize){
    print!("\x1B[{cols}C");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_left(cols:usize){
    print!("\x1B[{cols}D");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_linestart(){
    print!("\r");
    io::stdout().flush()
        .expect("Failed to flush");
}

pub fn hide_cursor(){
    print!("\x1B[?25l");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn show_cursor(){
    print!("\x1B[?25h");
    io::stdout().flush()
        .expect("Failed to flush");
}





// random stuff

pub fn figlet_figure(message:String)-> String{
    let font = figleter::FIGfont::standard()
                            .expect("Unable to retrieve the standard fonts");
    
    let figure = font.convert(&message);

    match figure {
        Some(_)=>{
            figure.unwrap().to_string()
        },
        None =>{
            return message;
        }
    }
}

