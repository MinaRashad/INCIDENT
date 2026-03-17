
// marking a contradiction will take the following steps

use std::{collections::{self, HashSet}, path::PathBuf, thread::sleep, time::Duration};

use crossterm::event::{Event, KeyCode, KeyEvent, poll, read};
use ratatui::{prelude::*, widgets::{Block, BorderType, Clear, List, ListState, Paragraph}};

use crate::{data::{self, docs::{self, Contradiction, Entry, Tag, add_contradiction, contradiction_exists}}, events};
use crate::game_state::GameState;
use crate::menu_components;
use crate::sound;
use crate::terminal;
use crate::views::docs::{DOCS_ROOT, choose_file};

pub fn mark_contradiction(first_doc:PathBuf) -> GameState {

    // decorations
    play_contradiction_animation();
    let sink = sound::play_forever(sound::SoundCategory::LowHumming)
                        .unwrap_or(rodio::Sink::new().0);

    // step 1: select second document
    let second_document = choose_file("Which document does it contradict?");

    // end humming music and start a new one
    drop(sink);

    let sink = sound::play_forever(sound::SoundCategory::LoudHumming)
                            .unwrap_or(rodio::Sink::new().0);
    
    sink.set_volume(0.3);

    let Some(second_document) = second_document
    else {return GameState::OpenPath(first_doc);};

    // step 2: Error checking
    // if same document
    if second_document == first_doc {
        print_error("The document does not contradict itself".to_string());
        return GameState::OpenPath(first_doc);
    }

    let (shared, first_tags, second_tags) = get_tags(
                     &Entry { path: first_doc.to_path_buf() },
                     &Entry { path: second_document.to_path_buf() });


    terminal::clear_screen();
    println!("{first_doc:?}");
    println!("{second_document:?}");
    println!("{first_tags:?}");
    println!("{second_tags:?}");
    menu_components::wait_for_input();

    // if they have nothing in common
    if shared.is_empty() {
        print_error("The documents have nothing in common!".to_string());
        return GameState::OpenPath(first_doc);
    }

    let contradicting_tag = select_tag(shared);

    // confirm difference
    let tag1 = first_tags
                            .iter()
                            .find(|t| t.id == contradicting_tag)
                            .map(|tag| tag.clone())
                            .unwrap_or_default();

    let tag2 = second_tags
                            .iter()
                            .find(|t| t.id == contradicting_tag)
                            .map(|tag| tag.clone())
                            .unwrap_or_default();
    
    // if they say the same thing
    if tag1.value == tag2.value {
        print_error(format!("The documents say the same thing: {}", tag1.value));
    }

    let contradiction = Contradiction{
        doc1:first_doc.to_path_buf(),
        doc2:second_document.to_path_buf(),
        disagree_on: tag1
    };

    // if it already exists
    if contradiction_exists(&contradiction) {
        print_error("This contradiction has already been found".to_string());
        return GameState::OpenPath(first_doc);
    }

    add_contradiction(contradiction);
    drop(sink);

    play_contradiction_found_animation();
    events::add_event(format!("contradiction;{};{};{};", 
                first_doc.file_name().unwrap_or_default().to_string_lossy(), 
                second_document.file_name().unwrap_or_default().to_string_lossy(),
                contradicting_tag
            ));

    GameState::OpenPath(first_doc)
}


fn print_error(text:String){
    sound::play(sound::SoundCategory::Error);
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
            .style(Style::new().bold().light_blue().on_light_green()))
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
    sound::play(sound::SoundCategory::GUIFeedback);
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





fn play_contradiction_animation() {
    use std::time::Instant;

    let mut renderer = ratatui::init();
    terminal::drain_input();

    let start = Instant::now();
    let mut last_play = Instant::now();
    let duration = sound::play(sound::SoundCategory::Bad);
    let offset = 100;
    while start.elapsed() < Duration::from_millis(1000) {
        let ms = start.elapsed().as_millis();
        
        // make sure to  play sound effects after each ends
        if let Some(dur) = duration
            && dur.as_millis() + offset<= last_play.elapsed().as_millis()
            {
                last_play = Instant::now();
                sound::play(sound::SoundCategory::Bad);
            }

        renderer.draw(|f| draw_contradiction_frame(f, ms)).unwrap();
        sleep(Duration::from_millis(33));
    }
}

fn draw_contradiction_frame(f: &mut Frame, ms: u128) {
    let area = f.area();
    let tick = (ms / 60) as usize;

    // cycle through dramatic colors
    let bg = match tick % 5 {
        0 => Color::Red,
        1 => Color::Rgb(220, 30, 30),
        2 => Color::Rgb(255, 180, 0),
        3 => Color::White,
        _ => Color::Rgb(180, 0, 0),
    };
    let fg = match bg {
        Color::White | Color::Rgb(255, 180, 0) => Color::Red,
        _ => Color::White,
    };
    let line_fg = match bg {
        Color::White => Color::Rgb(255, 150, 150),
        _ => Color::Rgb(120, 0, 0),
    };

    // cross-hatch speed lines background
    let w = area.width as usize;
    let bg_lines: Vec<Line> = (0..area.height)
        .map(|row| {
            let ch = if row % 2 == 0 { "╲ " } else { "╱ " };
            let s: String = ch.chars().cycle().take(w).collect();
            Line::from(Span::styled(s, Style::default().fg(line_fg).bg(bg)))
        })
        .collect();
    f.render_widget(Paragraph::new(Text::from(bg_lines)), area);

    // phase 2: text + corner decorations after 400ms
    if ms > 400 {
        let excl_style = Style::default().fg(fg).bg(bg).bold();
        let (w, h) = (area.width, area.height);
        for &(x, y) in &[
            (0u16, 0u16),
            (w.saturating_sub(2), 0),
            (0, h.saturating_sub(1)),
            (w.saturating_sub(2), h.saturating_sub(1)),
        ] {
            if x < w && y < h {
                f.render_widget(
                    Paragraph::new("!!").style(excl_style),
                    Rect::new(x, y, 2, 1),
                );
            }
        }

        let box_w = 32u16.min(area.width.saturating_sub(2));
        let box_h = 5u16;
        let bx = area.width.saturating_sub(box_w) / 2;
        let by = area.height.saturating_sub(box_h) / 2;
        let box_area = Rect::new(bx, by, box_w, box_h);

        f.render_widget(Clear, box_area);
        let text_style = Style::default()
            .fg(fg)
            .bg(bg)
            .bold()
            .add_modifier(Modifier::RAPID_BLINK);

        let block = Block::bordered()
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(fg).bg(bg).bold());

        let para = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("C O N T R A D I C T I O N !", text_style)),
        ]))
        .alignment(Alignment::Center)
        .block(block);

        f.render_widget(para, box_area);
    }
}

fn play_contradiction_found_animation() {
    use std::time::Instant;

    let mut renderer = ratatui::init();
    terminal::drain_input();

    let start = Instant::now();
    let mut last_play = Instant::now();
    let duration = sound::play(sound::SoundCategory::Good);
    let offset = 100;
    while start.elapsed() < Duration::from_millis(1000) {
        let ms = start.elapsed().as_millis();

        if let Some(dur) = duration
            && dur.as_millis() + offset <= last_play.elapsed().as_millis()
        {
            last_play = Instant::now();
            sound::play(sound::SoundCategory::Good);
        }

        renderer.draw(|f| draw_contradiction_found_frame(f, ms)).unwrap();
        sleep(Duration::from_millis(33));
    }
}

fn draw_contradiction_found_frame(f: &mut Frame, ms: u128) {
    let area = f.area();
    let tick = (ms / 60) as usize;

    // cycle through triumphant golds and whites
    let bg = match tick % 5 {
        0 => Color::Rgb(255, 200, 0),
        1 => Color::Rgb(255, 230, 100),
        2 => Color::White,
        3 => Color::Rgb(200, 160, 0),
        _ => Color::Rgb(255, 215, 0),
    };
    let fg = match bg {
        Color::White => Color::Rgb(180, 130, 0),
        _ => Color::Black,
    };
    let line_fg = match bg {
        Color::White => Color::Rgb(255, 220, 100),
        _ => Color::Rgb(200, 160, 0),
    };

    // radiating star-burst background
    let w = area.width as usize;
    let bg_lines: Vec<Line> = (0..area.height)
        .map(|row| {
            let ch = if row % 2 == 0 { "✦ " } else { "· " };
            let s: String = ch.chars().cycle().take(w).collect();
            Line::from(Span::styled(s, Style::default().fg(line_fg).bg(bg)))
        })
        .collect();
    f.render_widget(Paragraph::new(Text::from(bg_lines)), area);

    // phase 2: text + corner decorations after 400ms
    if ms > 400 {
        let star_style = Style::default().fg(fg).bg(bg).bold();
        let (w, h) = (area.width, area.height);
        for &(x, y) in &[
            (0u16, 0u16),
            (w.saturating_sub(2), 0),
            (0, h.saturating_sub(1)),
            (w.saturating_sub(2), h.saturating_sub(1)),
        ] {
            if x < w && y < h {
                f.render_widget(
                    Paragraph::new("★★").style(star_style),
                    Rect::new(x, y, 2, 1),
                );
            }
        }

        let box_w = 36u16.min(area.width.saturating_sub(2));
        let box_h = 5u16;
        let bx = area.width.saturating_sub(box_w) / 2;
        let by = area.height.saturating_sub(box_h) / 2;
        let box_area = Rect::new(bx, by, box_w, box_h);

        let text_style = Style::default()
            .fg(fg)
            .bg(bg)
            .bold()
            .add_modifier(Modifier::RAPID_BLINK);

        f.render_widget(Clear, box_area);

        let block = Block::bordered()
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(fg).bg(bg).bold());

        let para = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(Span::styled("C O N T R A D I C T I O N  F O U N D !", text_style)),
        ]))
        .alignment(Alignment::Center)
        .block(block);

        f.render_widget(para, box_area);
    }
}