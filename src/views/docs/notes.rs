use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers, poll, read, KeyEvent};
use ratatui::{prelude::*, widgets::{Block, BorderType, Borders, Paragraph, Wrap}};

use crate::data::docs::{Note, add_note};
use crate::game_state::GameState;
use crate::sound;
use crate::terminal;

enum Field {
    Title,
    Content,
}

struct NoteState {
    title: String,
    content: String,
    active_field: Field,
    running: bool,
    saved: bool,
    path: Option<PathBuf>,
}

impl NoteState {
    fn new(path: Option<PathBuf>) -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            active_field: Field::Title,
            running: true,
            saved: false,
            path,
        }
    }
}

pub fn write_note(path: Option<PathBuf>) -> GameState {
    let mut state = NoteState::new(path.clone());
    let mut terminal = ratatui::init();
    terminal::drain_input();

    while state.running {
        terminal.draw(|f| render_note(f, &state))
            .expect("failed to draw frame");

        if poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Event::Key(key) = read().expect("failed to read event") {
                handle_input(key, &mut state);
            }
        }
    }

    if state.saved {
        let note = Note {
            path: state.path.clone(),
            title: if state.title.is_empty() { "Untitled".to_string() } else { state.title.clone() },
            content: state.content.clone(),
        };
        add_note(note);
        sound::play(sound::SoundCategory::Good);
    }

    if let Some(p) = path {
        GameState::OpenPath(p)
    } else {
        GameState::GoBack(PathBuf::from(crate::views::docs::DOCS_ROOT))
    }
}

fn render_note(f: &mut Frame, state: &NoteState) {
    let area = f.area();
    let layout = Layout::vertical([
        Constraint::Length(3), // Instructions
        Constraint::Length(3), // Title input
        Constraint::Fill(1),   // Content input
        Constraint::Length(1), // Help
    ]).split(area);

    let instructions = Paragraph::new(format!(
        "Writing note for: {}",
        state.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "General".to_string())
    ))
    .block(Block::bordered().title("Note Context"))
    .alignment(Alignment::Center);

    let title_style = if matches!(state.active_field, Field::Title) {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let title_block = Block::bordered()
        .title("Title")
        .border_type(if matches!(state.active_field, Field::Title) { BorderType::Thick } else { BorderType::Plain })
        .border_style(title_style);

    let title_paragraph = Paragraph::new(state.title.as_str())
        .block(title_block);

    let content_style = if matches!(state.active_field, Field::Content) {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let content_block = Block::bordered()
        .title("Content")
        .border_type(if matches!(state.active_field, Field::Content) { BorderType::Thick } else { BorderType::Plain })
        .border_style(content_style);

    let content_paragraph = Paragraph::new(state.content.as_str())
        .block(content_block)
        .wrap(Wrap { trim: false });

    let help = Paragraph::new("[Tab] Switch Field  [Ctrl+S] Save  [Esc] Cancel")
        .centered()
        .add_modifier(Modifier::REVERSED);

    f.render_widget(instructions, layout[0]);
    f.render_widget(title_paragraph, layout[1]);
    f.render_widget(content_paragraph, layout[2]);
    f.render_widget(help, layout[3]);
}

fn handle_input(key: KeyEvent, state: &mut NoteState) {
    if key.kind == event::KeyEventKind::Release {
        return;
    }

    match key.code {
        KeyCode::Esc => {
            state.running = false;
        }
        KeyCode::Tab => {
            state.active_field = match state.active_field {
                Field::Title => Field::Content,
                Field::Content => Field::Title,
            };
            sound::play(sound::SoundCategory::GUIFeedback);
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if !state.title.trim().is_empty() || !state.content.trim().is_empty() {
                state.saved = true;
                state.running = false;
            } else {
                 sound::play(sound::SoundCategory::Error);
            }
        }
        KeyCode::Char(c) => {
            match state.active_field {
                Field::Title => {
                    if state.title.len() < 50 {
                        state.title.push(c);
                        sound::keystroke_play(c);
                    }
                }
                Field::Content => {
                    state.content.push(c);
                    sound::keystroke_play(c);
                }
            }
        }
        KeyCode::Backspace => {
            match state.active_field {
                Field::Title => { state.title.pop(); }
                Field::Content => { state.content.pop(); }
            }
            sound::play(sound::SoundCategory::Type);
        }
        KeyCode::Enter => {
            match state.active_field {
                Field::Title => {
                    state.active_field = Field::Content;
                }
                Field::Content => {
                    state.content.push('\n');
                }
            }
            sound::play(sound::SoundCategory::Space);
        }
        _ => {}
    }
}
