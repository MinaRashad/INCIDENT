use std::{io::{self, Write}, thread, time::Duration};

use crate::terminal;
use crate::sound;

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