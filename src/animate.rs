use std::{io::{self, Write, Error}, thread, time::Duration};

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
           let Some(duration) = sound::click(){
            thread::sleep(duration);
        }else{
            thread::sleep(Duration::from_millis(delay_ms));
        }
        
    }

}