
// marking a contradiction will take the following steps

use std::{collections::{self, HashSet}, path::PathBuf, thread::sleep, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEvent, poll, read};
use ratatui::{prelude::*, widgets::{Block, List, ListState, Paragraph}};

use crate::{data::{self, docs::{self, Entry, Tag}}, game_state::GameState, menu_components::{self, date}, terminal, views::docs::{DOCS_ROOT, choose_file}};

pub fn mark_contradiction(first_doc:PathBuf) -> GameState {

    let second_document = choose_file("Which document does it contradict?");

    let Some(second_document) = second_document
    else {return GameState::OpenPath(first_doc);};

    if second_document == first_doc {
        print_error("The document does not contradict itself".to_string());
        return GameState::OpenPath(first_doc);
    }

    

    let (shared, first_tags, second_tags) = get_tags(
                     &Entry { path: first_doc.to_path_buf() },
                     &Entry { path: second_document.to_path_buf() });

    if shared.is_empty() {
        print_error("The documents have nothing in common!".to_string());
        return GameState::OpenPath(first_doc);
    }

    let contradicting_tag = select_tag(shared);

    // confirm difference
    let val1 = first_tags
                            .iter()
                            .find(|t| t.id == contradicting_tag)
                            .map(|tag| tag.value.clone())
                            .unwrap_or_default();
    let val2 = second_tags
                            .iter()
                            .find(|t| t.id == contradicting_tag)
                            .map(|tag| tag.value.clone())
                            .unwrap_or_default();
    if val1 == val2 {
        print_error(format!("The documents say the same thing :{val1}"));

    }

    GameState::OpenPath(first_doc)
}


fn print_error(text:String){
    terminal::clear_screen();
    terminal::clear_scrollback();
    
    let title = terminal::figlet_figure("ERROR".to_string());
    let title = terminal::center_multiline(title);
    let title = terminal::foreground_color(title, [255, 50, 50]);
    
    println!("{}", title);
    println!();
    println!();
    
    let error_text = terminal::center_multiline(text.to_string());
    let error_text = terminal::bold(error_text);
    
    println!("{}", error_text);
    println!();
    
    menu_components::wait_for_input();
}

struct TagSelectionState {
    list: ListState,
    tags: Vec<(u32, String)>, // (id, name)
    selected: Option<u32>,
}

fn get_tags(path1: &Entry, path2:&Entry)
-> (Vec<u32>, Vec<Tag>, Vec<Tag>)
{
    let first_doc_tags = data::docs::get_tags_of(path1);
    let sec_doc_tags = data::docs::get_tags_of(path2);
    let mut first_ids = HashSet::new();
    for tag in &first_doc_tags{
        first_ids.insert(tag.id);
    }
    let mut second_ids = HashSet::new();
    for tag in &sec_doc_tags{
        second_ids.insert(tag.id);
    }; 

    let shared_ids: Vec<u32> = first_ids.intersection(&second_ids)
                        .map(|num| num.clone())
                        .collect();

    (shared_ids, first_doc_tags, sec_doc_tags)
}



fn select_tag(shared: Vec<u32>) -> u32 {
    let tags: Vec<(u32, String)> = shared.iter()
        .map(|&id| (id, docs::get_tag_name(id)))
        .collect();

    let mut state = TagSelectionState {
        list: ListState::default(),
        tags,
        selected: None,
    };
    state.list.select(Some(0));

    sleep(Duration::from_millis(100));

    let mut renderer = ratatui::init();
    terminal::drain_input();

    loop {
        renderer.draw(|f| select_tag_render(f, &mut state))
            .expect("failed to draw frame");

        if poll(Duration::from_millis(500)).unwrap_or(false) {
            match read() {
                Ok(Event::Key(k)) if k.is_release() => select_tag_input(k, &mut state),
                _ => {}
            }
        }

        if let Some(id) = state.selected {
            return id;
        }
    }
}

fn select_tag_render(f: &mut Frame, state: &mut TagSelectionState) {
    let area = f.area();
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).split(area);

    let title = Paragraph::new("What do they contradict on?")
        .block(Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .style(Style::new().bold().blue().on_light_green()))
        .rapid_blink()
        .alignment(Alignment::Center);

    let names: Vec<String> = state.tags.iter()
                        .map(|(_, name)| name.as_str())
                        .map(|name| name.replace("_", " "))
                        .collect();

    let list = List::new(names)
        .block(Block::bordered().title("Select Tag"))
        .style(Style::new().black().on_white())
        .highlight_style(Style::new().bold().red().on_white().rapid_blink());

    let help = Paragraph::new("[↑↓] navigate   [Enter] select")
        .centered()
        .add_modifier(Modifier::REVERSED);

    f.render_widget(title, layout[0]);
    f.render_stateful_widget(list, layout[1], &mut state.list);
    f.render_widget(help, layout[2]);
}

fn select_tag_input(k: KeyEvent, state: &mut TagSelectionState) {
    match k.code {
        KeyCode::Up => state.list.select_previous(),
        KeyCode::Down => state.list.select_next(),
        KeyCode::Enter => {
            let idx = state.list.selected().unwrap_or(0);
            if let Some(&(id, _)) = state.tags.get(idx) {
                state.selected = Some(id);
            }    
        }    
        _ => {}
    }    
}    


