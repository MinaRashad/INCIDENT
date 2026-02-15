use std::collections;
use std::ffi::OsString;
use std::io::Error;
use std::io::Read;
use std::path::{PathBuf,Path};
use std::fs;

use std::thread::sleep;
use std::time::Duration;


use toml::Value;

use crate::animate;
use crate::data;
use crate::data::docs::metadata;
use crate::data::{Metadata, MetadataField, Entry};
use crate::data::docs::update_metadata;
use crate::menu_components;
use crate::game_state::GameState;
use crate::terminal;
use crate::views::chat;


pub const DOCS_ROOT:&str = "assets/documents";
// static DOCS_METADATA = 


pub fn start() -> GameState {

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
    terminal::clear_scrollback();
    if path.is_dir(){
        open_dir(path)
    }else {
        open_file(path)
    }   
}

fn open_dir(path:PathBuf) -> GameState{
    // we come here from Docs,
    if !path.is_dir() {
        panic!("open_dir called on a file")
    }
    let contents = get_folder_contents(path.clone());
    let contents = match contents {
        Ok(result)=>result,
        Err(_)=>panic!("Unable to retireve root docs contents")
    };

    let contents = match get_options(contents) {
        Ok(v)=> v,
        Err(_)=> panic!("Failed to format options")
    };

    let title = path.to_str()
                        .and_then(|path| path.strip_prefix("assets"))
                        .unwrap_or("Documents");
    let options_gamestate: Vec<GameState> = contents
                         .iter()
                         .map(|option| path_gamestate(option))
                         .chain(
                            [if path != PathBuf::from(DOCS_ROOT) 
                                 {GameState::GoBack(
                                    PathBuf::from(path.parent()
                                    .and_then(|p|p.to_str())
                                    .unwrap_or(DOCS_ROOT)
                                        )
                                 )} 
                            else {GameState::Exit}]
                        )
                         .collect();
    

    menu_components::multichoice(title, options_gamestate, true)
}

fn path_gamestate(path:&PathBuf)->GameState{
    
    let entry = data::Entry{path: path.to_path_buf()};
    if let Some(metadata) = data::docs::metadata(&entry) &&
       let Some(player_access) = data::player::get_access_level() &&
       let Some(required_access) = metadata.access_level{
        if player_access < required_access as i32 {
            return GameState::Unauthorized(path.to_path_buf());
        }
        
    
    };
    
    GameState::OpenPath(path.to_path_buf())
}

fn open_file(path:PathBuf) -> GameState{
    if !path.is_file() {
        panic!("open file called on dir")
    };

    let entry = Entry{path:path.clone()};
    
    // open the file in the metadata
    update_metadata(&entry, MetadataField::Opened(true));

    let header_str = match name(&path) {
        Ok(s) => header(s),
        Err(_) => panic!("Unable to create the header")
    };

    let [w, h] = terminal::size();

    let mut file_content = String::new();
    let _ = fs::File::open(&path)
            .and_then(|mut file| file.read_to_string(&mut file_content));
    
    // the file content should be read
    // if its empty, lets add some text

    if file_content.is_empty(){
        file_content += "THIS FILE DOES NOT EXIST"
    }
    
    let file_content = file_content;

    let file_content = match embed_images_in(&file_content) {
        Some(text)=>text,
        None=> file_content
    };

    terminal::clear_scrollback();

    if file_content.starts_with("[CHAT]"){
        let mut lines = file_content.lines();
        let line = lines.next();
        let line = match line {
            Some(line)=>line,
            None => panic!("How come we cant get the first line")
        };
        let split = line.split("][");
        let split : Vec<&str> = split.into_iter().collect();
        let sender = split[1];
        let reciever = split[2];
        let reciever = match reciever.strip_suffix(']') {
            Some(r)=> r,
            None => reciever
        };

        chat::parse_and_display_chat(&file_content, (sender, reciever));
    }
    else{
        let file_content = header_str + file_content.as_str();
        animate::line_typer(&file_content, 50);
    }



    menu_components::wait_for_scroll();
    
    GameState::OpenPath(path.as_path()
                        .parent()
                        .and_then(|p| Some(p.to_path_buf()))
                        .unwrap_or(PathBuf::from(DOCS_ROOT)))
}

// Docs helper functions

fn docs_init(){
    // this function does any initiation needed on document side
    terminal::set_title("INCIDENT: Documents");
    // so I dont have to implement my own scroll feature
    terminal::exit_alternative_buffer();
    terminal::disable_text_warp();
    terminal::clear_screen();
    terminal::clear_scrollback();
    
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

fn get_options(contents:Vec<(bool, OsString, PathBuf)>)
->Result<Vec<PathBuf>, Error>{

    let options : Vec<PathBuf> = contents
                    .iter()
                    .map(|entry| entry.2.clone())
                    .collect();

    Ok(options)
}


// file functions
fn name(path:&Path)->Result<String, OsString>{
    let file_name = path.file_name();
    let file_name = match file_name {
        Some(name)=>name,
        None => panic!("Should be able to get the file name")
    };
    let file_name = file_name.to_os_string();
    let file_name = file_name.into_string()?;
    
    Ok(file_name)
}

fn header(file_name:String)->String{
    let mut result = String::new();

    result += "════════════════════════════════════════════════════════════\n";
    result += file_name.as_str();
    result += "\n════════════════════════════════════════════════════════════\n";

    terminal::center_multiline(result)
}

fn embed_images_in(file_content:&str)->Option<String>{
    let mut result = String::new();

    let [w, h] = terminal::size();
    let w= w as u32;
    let image_width = (w / 2).max(5);
    let image_height = image_width/2;

    let lines :Vec<&str> = file_content.lines().collect();
    let mut image_queue = collections::VecDeque::new();

    let mut i = 0;
    while i < lines.len(){
        let line = lines[i];
        if line.starts_with("<img>") {
            image_queue.push_back(line);
        }

        if !image_queue.is_empty() {
            let img_path = image_queue.pop_front()?;
            // replace the tag with the ascii art
            let img_path = img_path.strip_prefix("<img>")?;
            let img_path = img_path.strip_suffix("</img>")?;
            let img_path = PathBuf::from(img_path);
            let image = data::ImageDoc::image(img_path);
            let image = menu_components::display_image(image, 
                Some(image_width),
                Some(image_height))?;
            let image:Vec<&str> = image.lines().collect();

            // now we have the image string
            // we just need to advance until we either
            // run out of extra text or exceed the image
            let mut j = 0;
            let mut image_str = String::new();
            while j < image.len() && i < lines.len(){
                i += 1;
                if lines[i].starts_with("<img>"){
                    image_queue.push_back(line);
                    continue;
                }
                image_str += image[j];
                image_str += "  "; // just add space between
                image_str += lines[i];
                image_str += "\n";
                j += 1
            }
            result += image_str.as_str();
        }
        else{
            result += line;
        }
        result += "\n";

        i += 1;

    }

    Some(result)
}

