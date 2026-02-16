use std::{io::{self, Write}, thread, time::Duration};

use crate::{sound::SoundCategory, terminal};
use crate::sound;

/// Displays text character-by-character with a typewriter effect
/// 
/// # Arguments
/// * `msg` - The message to display
/// * `delay_ms` - Delay between characters in milliseconds (used only if sound is false)
/// * `sound` - If true, plays keystroke sounds and uses sound duration for timing
///             If false, uses delay_ms for timing
/// 
/// Characters are printed one at a time with delays between them.
/// When sound is enabled, keystroke sounds control the timing.
pub fn typer(msg:&str,delay_ms:u64, sound:bool){

    for c in msg.chars(){
        print!("{c}");
        match io::stdout().flush() {
            Ok(_) => {},
            Err(_) => panic!("Didnt flush")
        };
        if sound &&
           let Some(duration) = sound::keystroke_play(c){
            thread::sleep(duration);
        }else{
            thread::sleep(Duration::from_millis(delay_ms));
        }
        
    }

}

/// Displays text line-by-line with delays between each line
/// 
/// # Arguments
/// * `msg` - Multi-line message to display
/// * `delay_ms` - Delay between lines in milliseconds
/// 
/// Each line is printed completely, then waits before printing the next line.
/// Useful for displaying log-style or sequential text content.
pub fn line_typer(msg:&str,delay_ms:u64){

    let lines = msg.lines();
    // let curr_height: u32 = 0;
    for line in lines{
        println!("{line}");

        thread::sleep(Duration::from_millis(delay_ms));
    }

}


/// Displays an animated loading bar that fills progressively
/// 
/// # Arguments
/// * `delay_ms` - Delay between animation frames in milliseconds
/// 
/// Creates a 15-block loading bar using Unicode block characters (▏▎▍▌▋▊▉█)
/// Each block fills progressively from empty to full, creating a smooth animation.
/// The cursor is manipulated to animate in-place, then returns to line start when complete.
pub fn loading_bar(delay_ms:u64){
    let blocks = ['▏', '▎', '▍', '▌',
                             '▋', '▊', '▉', '█'];

    print!("Loading: ");
    
    let block_count = 15;
    let sub_block_size = 1;
    let block_size = sub_block_size*blocks.len();
    let max= block_count*block_size;
    let delay= Duration::from_millis(delay_ms);

    // now we can represent i as


    for i in 1..max{
        let n = i / block_size; // how many blocks are in i
        let remaining = i - n*block_size; 
        let k = remaining/sub_block_size;// which subblock

        if k == 0{
            terminal::move_cursor_right(1);
        }

        print!("{}",blocks[k]);
        terminal::move_cursor_left(1); // go back, no need to flush again
        
        std::thread::sleep(delay);
    };
    terminal::move_cursor_linestart();

}

/// Creates a flashing animation by alternating between showing and hiding content
/// 
/// # Arguments
/// * `content` - The content to flash on screen
/// * `delay_ms` - Delay for both visible and invisible states in milliseconds
/// * `count` - Number of times to flash (complete on/off cycles)
/// * `sound` - optional to indicate the sound the plays each cycle
/// 
/// Clears the screen, shows content, waits, clears screen again, waits.
/// Repeats this cycle `count` times creating a flashing effect.
/// Useful for warnings, alerts, or attention-grabbing displays.
pub fn flash(content:String, delay_ms:u64, count:usize, sound_name:Option<SoundCategory>){

    for i in 0..count{
        terminal::clear_screen();
        terminal::clear_scrollback();
        println!("{}",content);
        if let Some(name) = &sound_name{
                sound::play(name.clone());
        }
        std::thread::sleep(Duration::from_millis(delay_ms));
        
        terminal::clear_screen();
        terminal::clear_scrollback();
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}