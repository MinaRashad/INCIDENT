use std::process::{self, Child};
use std::io::{self, Stdin, Stdout};
use std::env;
// This module is to manage multiple
// windows. Nothing related to the windows
// operating system specifically

struct Window{
    name:String,
    pid:i32,
    input_stream:Stdin,
    output_streem:Stdout
}

#[cfg(target_os = "windows")]
const TERMINAL: &str = "cmd";

#[cfg(target_os = "macos")]
const TERMINAL: &str = "osascript";

#[cfg(target_os = "linux")]
const TERMINAL: &str = "gnome-terminal";

pub fn start_mode(name:&str){
    // name would be the assigned mode
    let exe_path = match env::current_exe() {
        Ok(x)=> x,
        Err(x)=> panic!("Unable to get exe path")
    };

    let exe_path = match exe_path.to_str() {
        Some(path)=> path,
        None=> panic!("Unable to get exe path")
    };

    let command = format!("{exe_path} --{name}");

    let _ = match TERMINAL {
        "cmd" =>{
            std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", &exe_path, &format!("--{name}")])
            .spawn()
        },
        "osascript" =>{
            std::process::Command::new("osascript")
                .args([
                    "-e", 
                    &format!("tell app \"Terminal\" to do script \"{} --{}\"", exe_path, name)
                ])
                .spawn()
        },
        "gnome-terminal" =>{
            std::process::Command::new("gnome-terminal")
            .args(["--", &exe_path, &format!("--{name}")])
            .spawn()
        },
        &_ =>{unimplemented!("not supported")}
    };
}