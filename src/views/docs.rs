use std::collections;
use std::env::current_dir;
use std::ffi::OsString;
use std::io::Error;
use std::io::Read;
use std::path::{PathBuf,Path};
use std::fs;

use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::vec;

use ansi_to_tui::IntoText;

use crossterm::event::KeyEvent;
use log::error;
use ratatui::symbols::block;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListState;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};
use crossterm::{
    event::{Event, KeyCode, poll, read},
};
use sha2::digest::crypto_common::Key;


use crate::data;
use crate::data::docs::ImageDoc;
use crate::data::docs::metadata;
use crate::data::docs::{MetadataField, Entry};
use crate::data::docs::update_metadata;
use crate::events::EventType;
use crate::menu_components;
use crate::game_state::GameState;
use crate::sound;
use crate::terminal;
use crate::util::parent;
use crate::views::chat;
use crate::events;
use crate::views::title;

pub mod contradictions;
pub mod notes;


pub const DOCS_ROOT:&str = "assets/documents";

/// Possible operations on documents
enum Operation {
    addNote,
    addContradiction
}


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


    menu_components::wait_for_input();
    docs_init();


    GameState::OpenPath(PathBuf::from(DOCS_ROOT))
}

pub fn open_path(path:PathBuf) -> GameState{
    terminal::clear_screen();
    terminal::clear_scrollback();

    // update metadata
    let entry = Entry{path:path.clone()};
    let file_metadata = data::docs::metadata(&entry);

    // if the file is not opened 
    if !file_metadata.opened {
        // mark metadata to open
        update_metadata(&entry, MetadataField::Opened(true));

        let event = EventType::OnPathOpen(path.clone());
        
        // add to events
        events::add_event(event.to_str());
    };

    
    
    // open the file in the metadata

    if path.is_dir(){
        open_dir(path)
    }else {
        if let Some(ext) = path.extension() &&
           let Some(ext) = ext.to_str() &&
           ["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&ext)
        {
            open_image(path)
        }
        else{
            open_file(path)
        }
    }   
}

fn open_dir(path:PathBuf) -> GameState{
    // we come here from Docs,
    let fallback = GameState::GoBack(PathBuf::from(DOCS_ROOT));
    if !path.is_dir() {
        error!("open_dir called on a file");
        return fallback;
    }
    let contents = get_folder_contents(path.clone());
    let contents = match contents {
        Ok(result)=>result,
        Err(_)=>{
            error!("Failed to get folder contents");
            return fallback;
        }
    };

    let mut contents = match get_options(contents) {
        Ok(v)=> v,
        Err(_)=> {
            error!("Failed to generate options from folder contents");
            return fallback;
        }
    };

    contents.sort_by_key(|p| !p.is_dir());


    let title = path.to_str()
                        .and_then(|path| path.strip_prefix("assets"))
                        .unwrap_or("Documents");
    let options_gamestate: Vec<GameState> = contents
                         .iter()
                         .map(|option| path_gamestate(option))
                         .chain(
                            [if path != PathBuf::from(DOCS_ROOT) 
                                 {GameState::GoBack(
                                    parent(path.clone())
                                 )} 
                            else {GameState::Exit}]
                        )
                         .collect();
    

    menu_components::multichoice(title, options_gamestate, true)
}

fn path_gamestate(path:&PathBuf)->GameState{

    let entry = Entry{path: path.to_path_buf()};
    let metadata = data::docs::metadata(&entry);

    if let Some(player_access) = data::player::get_access_level() {


        if let Some(required_access) = metadata.access_level &&
         player_access < required_access as i32 
         {
                return GameState::Unauthorized(path.to_path_buf());
        }

        if let Some(_password) = metadata.password && 
           ! metadata.opened
        {
                return GameState::PasswordProtected(path.to_path_buf());
        }
        
        
    
    };
    
    GameState::OpenPath(path.to_path_buf())
}

fn open_file(path:PathBuf) -> GameState{
    if !path.is_file() {
        error!("open_file called on a nonfile");
        return GameState::GoBack(PathBuf::from(DOCS_ROOT));

    };
    
    let header_str = match name(&path) {
        Ok(s) => header(s),
        Err(_) => {
            error!("open_file called on a nonfile");
            return GameState::GoBack(PathBuf::from(DOCS_ROOT));
        }
    };


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

    let file_content = if file_content.starts_with("[CHAT]"){
                chat::parse_chat(&file_content)
            }else{
                file_content
            };

    
    let file_content = header_str + file_content.as_str();
    if let Some(operation) = document_view(&file_content, 100){
        return match operation {
            Operation::addContradiction => GameState::Contradiction(path),
            Operation::addNote => GameState::Note(Some(path))
        }
    };
    
    
    GameState::GoBack(parent(path))
}

fn open_image(path: PathBuf) -> GameState{
    terminal::clear_screen();
    terminal::clear_scrollback();

    let img = ImageDoc(path.clone());
    let [w, _h] = terminal::size();    

    let max_width = (w*9)/10;

    let img = menu_components::display_image(
        img,
        Some(max_width as u32),
         None);

    
    let op = match img {
        Some(img) => document_view(img.as_str(), 50),
        None => None
    };

    if let Some(op) = op 
    {
        return match op {
            Operation::addContradiction => GameState::Contradiction(path),
            Operation::addNote => GameState::Note(Some(path))
        }
    };
    

    GameState::GoBack(parent(path))
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


    let result: Vec<(bool, OsString, PathBuf)> = folder
                        .map(|entry| match entry {
                        Ok(entry)=> Some(entry),
                        Err(_) => {
                            error!("Failed ro read a file");
                            return None
                        }                       
                    })
                    .filter(|entry| entry.is_some())
                    .map(|entry| entry.unwrap())
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
            let image = ImageDoc::image(img_path);
            let image = menu_components::display_image(image, 
                Some(image_width),
                Some(image_height))?;
            let image:Vec<&str> = image.lines().collect();

            // now we have the image string
            // we just need to advance until we either
            // run out of extra text or exceed the image
            let mut j = 0;
            let mut image_str = String::new();
            while j < image.len() && i < lines.len()-1{
                println!("=>{j},{i}, {len}", len=lines.len());
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
            // after embedding the lines, if the image is not done yet,
            // add the rest of the lines
            while j < image.len() {
                image_str += image[j];
                image_str += "\n";
                j += 1;
            };
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




/// Function to show the document
/// It can also indicate the user wants to 
/// add a note or add a contradiction to the case files
fn document_view(msg:&str,delay_ms:u64) -> Option<Operation> {
    let lines:Vec<&str> = msg.lines().collect();
    // let curr_height: u32 = 0;
    let mut terminal = ratatui::init();
    let mut start: usize = 0;
    loop{
        // render
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(
                    [
                        Constraint::Fill(1),
                        Constraint::Length(1)
                    ]
                )
                .split(area);
            let height: usize = area.height as usize;
            

            let help_text = Paragraph::new("[Enter] exit   [↑↓] scroll   [C] Contradiction   [N] Note")
                        .centered().add_modifier(Modifier::REVERSED);
            
            let content = lines.iter().skip(start).take(height)
                                .map(|line|
                                     (*line).into_text().unwrap_or_default())
                                .fold(Text::from(""),
                                |acc, curr| acc + curr);
            
            frame.render_widget(content, layout[0]);
            frame.render_widget(help_text, layout[1]);
        })
        .expect("Failed to render");

        //input
        if poll(Duration::from_millis(delay_ms)).unwrap_or(false)
        {
            if let Event::Key(k) = read().ok()? 
            && k.is_release() 
            {
                if (KeyCode::is_enter(&k.code) || KeyCode::is_esc(&k.code)){
                    break None;
                }
                // scrolling feature
                else if  KeyCode::is_up(&k.code){
                    if start > 0{
                        start = start - 1;
                    }
                } else if KeyCode::is_down(&k.code) {
                    if start < lines.len(){
                        start = start + 1
                    }
                }
                // Contradition
                else if KeyCode::is_char(&k.code, 'c'){
                    break Some(Operation::addContradiction);
                }
                // Note
                else if KeyCode::is_char(&k.code, 'n'){
                    break Some(Operation::addNote);
                }
            };
        }
        
    }

}

struct DocSelectionState  {
        curr_path: PathBuf,
        curr_selection: ListState, 
        running: bool,
        children: Vec<PathBuf>,
        title:String
}

/// # `choose_file`
/// a general function that prompts the user to select a file
/// This is different from the regular directory 

pub fn choose_file(title:&str)-> Option<PathBuf> 
{
    let mut state = DocSelectionState {
        curr_path: PathBuf::from(DOCS_ROOT),
        curr_selection: ListState::default(),
        running: true,
        children: vec![],
        title: title.to_string()
    };

    let mut renderer = ratatui::init();    

    terminal::drain_input();
    loop {
        if state.curr_path.is_file(){
            break Some(state.curr_path);
        }
        if !state.running{
            break None;
        }
        // render
        renderer.draw(
            |f| 
                    {choose_file_render(f, &mut state);}
                ).expect("failed to draw a frame");


        // input
        if poll(Duration::from_millis(500)).unwrap_or(false){
            // input goes here
            match read() {
                Ok(Event::Key(k))
                if k.is_release() => choose_file_input(k, &mut state),            
                _ => {} // do nothing if anything else including error
            };

        }
    }
}

fn choose_file_render(f: &mut Frame, state: &mut DocSelectionState)
{

    let curr_path = &state.curr_path;
    let curr_path_str = &curr_path.to_string_lossy()
                            .replace("\\", "/");
    let curr_path_str = curr_path_str
                            .strip_prefix("assets")
                            .unwrap_or("Documents");

    let mut children = get_unopend_children(curr_path);
    children.sort_by_key(|c| !c.is_dir());
    state.children = children;

    let names = (&state.children)
                .iter()
                .filter_map(|path| {
                    let name = path.file_name()?.to_string_lossy();
                    Some(format!("{}{}", name, if path.is_dir() { "/" } else { "" }))
                });
    
    // now we have an iterator over opened children
    // render them in a stateful widget
    let area = f.area();
    let layout = Layout::vertical([
                Constraint::Length(3),// Title
                Constraint::Fill(1),  // main content
                Constraint::Length(1) // Help text
            ]).split(area);

    let title = 
            Paragraph::new("Where is the Contradiction")
            .block(Block::bordered()
                    .border_set(symbols::border::ROUNDED)
                    .style(Style::new().red().on_white()))
            .rapid_blink()
            .alignment(Alignment::Center)
            ;

    let selection_menu = List::new(names)
                    .block(Block::bordered().title(curr_path_str))
                    .style(Style::new().black().on_white())
                    .highlight_style(
                        Style::new().bold().red().on_white().rapid_blink()
                    );


    let help = Paragraph::new("[↑↓] navigate   [Enter] open   [Esc] cancel")
        .centered()
        .add_modifier(Modifier::REVERSED);

    f.render_widget(title, layout[0]);
    f.render_stateful_widget(selection_menu, layout[1], &mut state.curr_selection);
    f.render_widget(help, layout[2]);

}

fn choose_file_input(k: KeyEvent, state: &mut DocSelectionState){
    sound::play(sound::SoundCategory::GUIFeedback);
    match k.code {
        KeyCode::Esc => {state.running = false},
        KeyCode::Up => {state.curr_selection.select_previous();},
        KeyCode::Down => {state.curr_selection.select_next();},
        KeyCode::Enter => {
            let selected_idx = state.curr_selection.selected().unwrap_or(0);
            let selected_path = state.children.get(selected_idx)
                        .unwrap_or(&state.children[0]);
            state.curr_selection.select(Some(0));
            state.curr_path = selected_path.to_path_buf();

            // println!("{}", state.curr_path.to_string_lossy());
            // thread::sleep(Duration::from_secs(1));


        }
        _ => {}
        
    }
}

fn get_unopend_children(path: &PathBuf)
-> Vec<PathBuf>
{
    // get the open files/folders from the current path
    if let Ok(dir) = path.read_dir() {
        return dir
            .filter (|entry| entry.is_ok())
            .map(|entry| Entry{path:entry.unwrap().path()})
            .filter(|entry| metadata(entry).opened)
            .map(|entry| entry.path)
            .collect()// only show opened;
    }

    return vec![];
        
}

