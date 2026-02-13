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
pub const CSI :&str="\x1B[";

// terminal utility
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

pub fn clear_screen(){
    print!("{CSI}2J{CSI}1;1H");
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
    format!("{CSI}38;2;{R};{G};{B}m{text}{CSI}0m", 
        R = color[0],
        G = color[1],
        B = color[2])
}

pub fn background_color(text:String,color:[u8;3])->String{
    format!("{CSI}48;2;{R};{G};{B}m{text}{CSI}0m", 
        R = color[0],
        G = color[1],
        B = color[2])
}

// format
pub fn bold(text:String) -> String{
    format!("{CSI}1m{text}{CSI}0m")
}

pub fn invert(text:String) -> String{
    format!("{CSI}7m{text}{CSI}0m")
}

// animation
pub fn blink(text:String) -> String{
    format!("{CSI}5m{text}{CSI}0m")
}


// positioning
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
    
    format!("{CSI}{space}G{text}")
}

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
pub fn get_input_now() -> KeyCode {
    let mut input = KeyCode::Null;
    crossterm::terminal::enable_raw_mode().
        expect("Failed to enter raw mode");

    // drain all queued inputs
    while poll(Duration::from_millis(0))
         .is_ok_and(|x| x) 
    {
        let _ = read();
    }
    
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

// blocking version 
pub fn get_input() -> KeyCode{
    let mut input = KeyCode::Null;

    // drain all queued inputs
    while poll(Duration::from_millis(0))
         .is_ok_and(|x| x)  {
        let _ = read();
    }

    match read() {
                Ok(Event::Key(event))=>{
                    input = event.code;
                },
                Ok(_) =>{}
                Err(_) =>{}
            }

    input
}

// cursor
pub fn move_cursor_up(rows:usize){
    print!("{CSI}{rows}A");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_down(rows:usize){
    print!("{CSI}{rows}B");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_right(cols:usize){
    print!("{CSI}{cols}C");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_left(cols:usize){
    print!("{CSI}{cols}D");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn move_cursor_linestart(){
    print!("\r");
    io::stdout().flush()
        .expect("Failed to flush");
}

pub fn hide_cursor(){
    print!("{CSI}?25l");
    io::stdout().flush()
        .expect("Failed to flush");
}
pub fn show_cursor(){
    print!("{CSI}?25h");
    io::stdout().flush()
        .expect("Failed to flush");
}


// alternative buffer
// for better screen cleanup

pub fn enter_alternative_buffer(){
    print!("{CSI}?1049h");
    io::stdout().flush()
        .expect("Failed to flush");
}

pub fn exit_alternative_buffer(){
    print!("{CSI}?1049l");
    io::stdout().flush()
        .expect("Failed to flush");
}



// random stuff

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

