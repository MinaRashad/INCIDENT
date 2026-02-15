use std::io::{Stdin, Stdout};
use std::env;
use std::os::windows::process::CommandExt;

// This module is to manage multiple
// windows. Nothing related to the windows
// operating system specifically

/// Represents a terminal window instance
/// Contains process information and I/O streams for the window
/// Currently unused - structure for future multi-window management
struct Window{
    /// Display name of the window/mode
    name:String,
    /// Process ID of the window
    pid:i32,
    /// Standard input stream for the window
    input_stream:Stdin,
    /// Standard output stream for the window
    output_streem:Stdout
}

/// Terminal emulator command for Windows
#[cfg(target_os = "windows")]
const TERMINAL: &str = "cmd";

/// Terminal emulator command for macOS
#[cfg(target_os = "macos")]
const TERMINAL: &str = "osascript";

/// Terminal emulator command for Linux
#[cfg(target_os = "linux")]
const TERMINAL: &str = "gnome-terminal";

/// Spawns a new terminal window running the current executable in a specific mode
/// 
/// # Arguments
/// * `name` - The mode name to pass as a command-line argument (e.g., "chat", "docs")
/// 
/// The new window will run the same executable with `--{name}` as an argument.
/// 
/// # Platform-specific behavior
/// - **Windows**: Uses `cmd` with `/C start` and detached mode flag
/// - **macOS**: Uses AppleScript to open a new Terminal.app window
/// - **Linux**: Uses `gnome-terminal` (requires GNOME desktop environment)
/// 
/// # Panics
/// Panics if the current executable path cannot be determined or converted to a string
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


    let _ = match TERMINAL {
        "cmd" =>{
            std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", exe_path, &format!("--{name}")])
            .creation_flags(0x00000008) // Detached mode
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
            .args(["--", exe_path, &format!("--{name}")])
            .spawn()
        },
        &_ =>{unimplemented!("not supported")}
    };
}