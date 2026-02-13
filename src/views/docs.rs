use core::error;
use std::ffi::OsString;
use std::io::Error;
use std::path::PathBuf;
use std::fs;

use std::thread::sleep;
use std::time::Duration;


use crate::data::Folder;
use crate::data::docs;
use crate::menu_components;
use crate::GameState;
use crate::terminal;


const DOCS_ROOT:&str = "assets/documents";


pub fn start() -> GameState {
    terminal::clear_screen();

    let header = terminal::bold(
        terminal::foreground_color(
            "DOCUMENT ARCHIVE TERMINAL".to_string(),
            [0, 200, 255],
        ),
    );

    println!("\n{}\n", header);

    println!("Initializing document index...");
    sleep(Duration::from_millis(300));

    println!("Verifying file integrity...");
    sleep(Duration::from_millis(350));

    let status = terminal::foreground_color(
        "[OK] Archive mounted".to_string(),
        [0, 255, 0],
    );
    println!("{}", status);

    sleep(Duration::from_millis(250));

    let ready = terminal::bold(
        terminal::foreground_color(
            "DOCUMENTS READY".to_string(),
            [255, 255, 255],
        ),
    );

    println!("\n{}\n", ready);

    docs_init();

    menu_components::wait_for_input();
    

    GameState::OpenPath(PathBuf::from(DOCS_ROOT))
}

pub fn open_path(path:PathBuf) -> GameState{
    terminal::clear_screen();
    if path.is_dir(){
        open_dir(path)
    }else {
        open_file(path)
    }   
}

fn open_dir(path:PathBuf) -> GameState{
    // we come here from Docs,
    let contents = get_folder_contents(path.clone());
    let contents = match contents {
        Ok(result)=>result,
        Err(_)=>panic!("Unable to retireve root docs contents")
    };

    let (contents, options) = match get_formatted_options(contents) {
        Ok((c,o))=> (c,o),
        Err(_)=> panic!("Failed to format options")
    };

    let title = path.to_str()
                        .and_then(|path| path.strip_prefix("assets"))
                        .unwrap_or("Documents");
    let options_str = options
                         .iter()
                         .map(|option| option.as_str())
                         .chain(["Back"])
                         .collect();
    

    let selection = menu_components::multichoice(title, options_str, true);

    match selection {
        i if i < options.len() => {GameState::OpenPath(
            contents[i].2.clone()
        )},
        _ => match path {
            p if p == PathBuf::from(DOCS_ROOT) => GameState::Exit,
            p => GameState::OpenPath(
                p.as_path().parent()
                .and_then(|p| Some(p.to_path_buf()))
                .unwrap_or(PathBuf::from(DOCS_ROOT))
            )
        }
    }
}

fn open_file(path:PathBuf) -> GameState{
    todo!()
}

// Docs helper functions

fn docs_init(){
    // this function does any initiation needed on document side
    terminal::set_title("INCIDENT: Documents");
}

fn get_folder_contents(path:PathBuf)  
-> Result<Vec<(bool, OsString, PathBuf)>, Error> {
    let folder = fs::read_dir(&path)?;


    let result: Vec<(bool, OsString, PathBuf)> = folder.map(|entry| match entry {
                        Ok(entry)=> entry,
                        Err(_) => panic!("Something unexpected happened")                        
                    })
                    .map(|entry| (entry.file_name(), entry.path()))
                    .map(|entry| (entry.1.is_dir(), entry.0, entry.1))
                    .into_iter()
                    .collect();

    return Ok(result);
}

fn get_formatted_options(contents:Vec<(bool, OsString, PathBuf)>)
->Result<(Vec<(bool, OsString, PathBuf)>, Vec<String>), Error>{

    let options : Vec<String> = contents
                    .iter()
                    .map(|entry| 
                    format!("{entry_type}{name}", 
                    entry_type = if entry.0 {""} else {"FILE - "},
                    name=  entry.1.to_str().unwrap() ))
                    .collect();

    
    Ok((contents, options))
}