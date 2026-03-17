use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, poll, read, KeyEvent};
use ratatui::{prelude::*, widgets::{Block, BorderType, Borders, Paragraph, Wrap, Clear}};

use crate::data::docs::{self, PlayerDocument, get_player_documents};
use crate::game_state::GameState;
use crate::sound;
use crate::terminal;

struct FindingsState {
    documents: Vec<PlayerDocument>,
    selected_idx: usize,
    running: bool,
    show_detail: bool,
}

impl FindingsState {
    fn new() -> Self {
        Self {
            documents: get_player_documents(),
            selected_idx: 0,
            running: true,
            show_detail: false,
        }
    }

    fn next(&mut self) {
        if self.documents.is_empty() { return; }
        self.selected_idx = (self.selected_idx + 1) % self.documents.len();
    }

    fn previous(&mut self) {
        if self.documents.is_empty() { return; }
        if self.selected_idx == 0 {
            self.selected_idx = self.documents.len() - 1;
        } else {
            self.selected_idx -= 1;
        }
    }

    // Grid navigation helpers
    fn move_right(&mut self, cols: usize) {
        if self.documents.is_empty() { return; }
        self.next();
    }

    fn move_left(&mut self, cols: usize) {
        if self.documents.is_empty() { return; }
        self.previous();
    }

    fn move_down(&mut self, cols: usize) {
        if self.documents.is_empty() { return; }
        let next = self.selected_idx + cols;
        if next < self.documents.len() {
            self.selected_idx = next;
        }
    }

    fn move_up(&mut self, cols: usize) {
        if self.documents.is_empty() { return; }
        if self.selected_idx >= cols {
            self.selected_idx -= cols;
        }
    }
}

pub fn display_findings() -> GameState {
    let mut state = FindingsState::new();
    let mut terminal = ratatui::init();
    terminal::drain_input();

    while state.running {
        terminal.draw(|f| render_findings(f, &state))
            .expect("failed to draw frame");

        if poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Event::Key(key) = read().expect("failed to read event") {
                handle_input(key, &mut state);
            }
        }
    }

    GameState::MainConsole
}

fn render_findings(f: &mut Frame, state: &FindingsState) {
    let area = f.area();
    
    let main_layout = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Fill(1),   // Grid
        Constraint::Length(1), // Help
    ]).split(area);

    let title = Paragraph::new("CASE FINDINGS & EVIDENCE")
        .block(Block::bordered().border_type(BorderType::Double))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    f.render_widget(title, main_layout[0]);

    let help = Paragraph::new("[Arrows] Navigate  [Enter] View Details  [Esc] Back")
        .centered()
        .add_modifier(Modifier::REVERSED);
    f.render_widget(help, main_layout[2]);

    if state.documents.is_empty() {
        let empty_msg = Paragraph::new("No findings recorded yet.\n\nUse 'N' in document view to add notes\nor identify contradictions.")
            .centered()
            .block(Block::bordered());
        f.render_widget(empty_msg, main_layout[1]);
        return;
    }

    // Grid layout
    let cols = 3;
    let rows = (state.documents.len() as f32 / cols as f32).ceil() as usize;
    
    let row_constraints = vec![Constraint::Length(8); rows];
    let grid_rows = Layout::vertical(row_constraints).split(main_layout[1]);

    for (r, row_area) in grid_rows.iter().enumerate() {
        let col_constraints = vec![Constraint::Percentage(100 / cols as u16); cols];
        let cells = Layout::horizontal(col_constraints).split(*row_area);

        for (c, cell_area) in cells.iter().enumerate() {
            let idx = r * cols + c;
            if idx < state.documents.len() {
                render_tile(f, *cell_area, &state.documents[idx], idx == state.selected_idx);
            }
        }
    }

    if state.show_detail {
        render_detail_overlay(f, &state.documents[state.selected_idx]);
    }
}

fn render_tile(f: &mut Frame, area: Rect, doc: &PlayerDocument, is_selected: bool) {
    let style = if is_selected {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let block = Block::bordered()
        .border_type(if is_selected { BorderType::Thick } else { BorderType::Plain })
        .border_style(style);

    match doc {
        PlayerDocument::Note(note) => {
            let preview = if note.content.len() > 60 {
                format!("{}...", &note.content[..60])
            } else {
                note.content.clone()
            };
            let text = format!("NOTE: {}\n\n{}", note.title, preview);
            let p = Paragraph::new(text)
                .block(block.title(" Note "))
                .wrap(Wrap { trim: true });
            f.render_widget(p, area);
        }
        PlayerDocument::Contradiction(contra) => {
            let doc1_name = contra.doc1.file_name().unwrap_or_default().to_string_lossy();
            let doc2_name = contra.doc2.file_name().unwrap_or_default().to_string_lossy();
            let text = format!("CONTRADICTION\n\nDocs: {}\n  vs  {}\nTag: {}", doc1_name, doc2_name, contra.disagree_on.value);
            let p = Paragraph::new(text)
                .block(block.title(" Contradiction ").border_style(if is_selected { style } else { Style::default().fg(Color::Red) }))
                .wrap(Wrap { trim: true });
            f.render_widget(p, area);
        }
    }
}

fn render_detail_overlay(f: &mut Frame, doc: &PlayerDocument) {
    let area = f.area();
    let popup_area = Rect::new(
        area.width / 10,
        area.height / 10,
        area.width * 8 / 10,
        area.height * 8 / 10,
    );

    f.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .style(Style::default().bg(Color::Black));

    match doc {
        PlayerDocument::Note(note) => {
            let content = format!(
                "TITLE: {}\nSOURCE: {}\n\n{}",
                note.title,
                note.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "General".to_string()),
                note.content
            );
            let p = Paragraph::new(content)
                .block(block.title(" Note Detail "))
                .wrap(Wrap { trim: false });
            f.render_widget(p, popup_area);
        }
        PlayerDocument::Contradiction(contra) => {
            let content = format!(
                "CONTRADICTION FOUND\n\nDocument A: {}\nDocument B: {}\n\nDisagreement on: {}",
                contra.doc1.to_string_lossy(),
                contra.doc2.to_string_lossy(),
                docs::get_tag_name(contra.disagree_on.id)
            );
            let p = Paragraph::new(content)
                .block(block.title(" Contradiction Detail ").border_style(Style::default().fg(Color::Red)))
                .wrap(Wrap { trim: true })
                .centered();
            f.render_widget(p, popup_area);
        }
    }
}

fn handle_input(key: KeyEvent, state: &mut FindingsState) {
    if key.kind == event::KeyEventKind::Release {
        return;
    }

    if state.show_detail {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                state.show_detail = false;
                sound::play(sound::SoundCategory::GUIFeedback);
            }
            _ => {}
        }
        return;
    }

    let cols = 3;
    match key.code {
        KeyCode::Esc => {
            state.running = false;
        }
        KeyCode::Right => {
            state.move_right(cols);
            sound::play(sound::SoundCategory::GUIFeedback);
        }
        KeyCode::Left => {
            state.move_left(cols);
            sound::play(sound::SoundCategory::GUIFeedback);
        }
        KeyCode::Down => {
            state.move_down(cols);
            sound::play(sound::SoundCategory::GUIFeedback);
        }
        KeyCode::Up => {
            state.move_up(cols);
            sound::play(sound::SoundCategory::GUIFeedback);
        }
        KeyCode::Enter => {
            if !state.documents.is_empty() {
                state.show_detail = true;
                sound::play(sound::SoundCategory::Good);
            }
        }
        _ => {}
    }
}
