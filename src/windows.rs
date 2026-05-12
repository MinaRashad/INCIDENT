// This module is to manage multiple
// windows. Nothing related to the windows
// operating system specifically.
//
// On desktop platforms a "window" is a freshly spawned OS terminal running
// this same executable with `--{mode}`. On wasm there is no process model,
// so the wasm build asks the browser host to open a new browser window that
// boots a fresh game instance in the requested mode (see `crate::wasm_host`).

/// Spawns a new "window" running the current executable in a specific mode.
///
/// # Arguments
/// * `name` - The mode name passed as a command-line argument (e.g. "docs", "chats").
///
/// The new window runs the same executable with `--{name}` as an argument.
#[cfg(not(target_arch = "wasm32"))]
pub fn start_mode(name: &str) {
    use std::env;

    let exe_path = match env::current_exe() {
        Ok(x) => x,
        Err(err) => panic!("Unable to get exe path: {err}"),
    };

    let exe_path = match exe_path.to_str() {
        Some(path) => path,
        None => panic!("Unable to get exe path"),
    };

    let arg = format!("--{name}");
    let _ = spawn_terminal(exe_path, &arg);
}

/// Windows: open a detached `cmd` window running the executable.
#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
fn spawn_terminal(exe_path: &str, arg: &str) -> std::io::Result<std::process::Child> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", exe_path, arg])
        .creation_flags(0x00000008) // DETACHED_PROCESS
        .spawn()
}

/// macOS: ask Terminal.app (via AppleScript) to run the executable in a new window.
#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn spawn_terminal(exe_path: &str, arg: &str) -> std::io::Result<std::process::Child> {
    std::process::Command::new("osascript")
        .args([
            "-e",
            &format!("tell app \"Terminal\" to do script \"{exe_path} {arg}\""),
        ])
        .spawn()
}

/// Other Unixes: try a few common terminal emulators.
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "windows"),
    not(target_os = "macos")
))]
fn spawn_terminal(exe_path: &str, arg: &str) -> std::io::Result<std::process::Child> {
    const CANDIDATES: &[&str] = &[
        "x-terminal-emulator",
        "gnome-terminal",
        "konsole",
        "xterm",
    ];
    let mut last_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no terminal emulator");
    for term in CANDIDATES {
        let result = match *term {
            "gnome-terminal" => std::process::Command::new(term)
                .args(["--", exe_path, arg])
                .spawn(),
            _ => std::process::Command::new(term)
                .args(["-e", exe_path, arg])
                .spawn(),
        };
        match result {
            Ok(child) => return Ok(child),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

/// wasm: there is no process model. Ask the browser host to open a new
/// game window in the requested mode.
#[cfg(target_arch = "wasm32")]
pub fn start_mode(name: &str) {
    crate::wasm_host::open_window(name);
}
