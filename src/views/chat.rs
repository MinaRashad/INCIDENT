use std::thread;
use std::time::Duration;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::poll;
use crossterm::event::read;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::BorderType;



use crate::menu_components;
use crate::GameState;
use crate::data::player;
use crate::sound;
use crate::terminal;

use crate::data;
use crate::data::chat::{ChatAppState, NPC, ChatLog, Message};

/// Entry point for the chat log viewer
/// Waits for user input before returning to the Chats state
/// 
/// # Returns
/// Always returns GameState::Chats
pub fn start()->GameState{

    let mut terminal = ratatui::init();
    
    let mut chatApp_state: ChatAppState = ChatAppState::default();
    chatApp_state.current_selection.select(Some(0));
    loop {
        // render chat
        terminal.draw(|f| render_chat(f, &mut chatApp_state))
            .expect("failed to draw a frame");

        // input
        if poll(Duration::from_millis(500)).is_ok(){
            // input goes here
            match read() {
                Ok(Event::Key(k))
                if k.is_release() => 
                {
                    sound::play(sound::SoundCategory::GUIFeedback);
                    match k.code {
                    KeyCode::Esc => break,
                    KeyCode::Down => {
                        sound::play(sound::SoundCategory::GUIFeedback);
                        chatApp_state.current_selection.select_next();
                    },
                    KeyCode::Up => {
                        sound::play(sound::SoundCategory::GUIFeedback);
                        chatApp_state.current_selection.select_previous();
                    },
                    _ => {}
                }}
                _ => {} // do nothing if anything else including error
            };
        }


    }

    ratatui::restore();
    GameState::Exit
}

fn render_chat(frame: &mut Frame, chat_app_state:&mut ChatAppState){

    let frame_area = frame.area();

    let general_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(5), // header
                            Constraint::Fill(1), // app content
                            Constraint::Length(1) // help
                        ]
                    ).split(frame_area);
    
    let app_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(39), // Chats
                            Constraint::Fill(1) // chat area
                        ]
                    ).split(general_layout[1]);

    let header = Paragraph::new("Whats'agram")
                .centered()
                .block(
                    Block::default().border_type(BorderType::Double)
                    .borders(Borders::ALL)
                );



    let current_chat = Block::bordered();
    

    let help = Paragraph::new("[ESC] close  [↑↓] navigate  [ENTER] open")
                .centered().reversed();

    
    frame.render_widget(header, general_layout[0]);
    frame.render_widget(help, general_layout[2]);
    frame.render_widget(current_chat, app_layout[1]);

    render_chatlogs(frame, 
        app_layout[0],
         chat_app_state);
}

fn render_chatlogs(frame: &mut Frame, chatlog_area:Rect,
     chatlog_state:&mut ChatAppState){
    
    let items: Vec<ListItem> = NPC::ALL.iter()
                    .map(|npc| format!("👤 {:?}", npc))
                    .map(|npc_name| 
                        ListItem::new(npc_name)
                        
                    )
                    .collect();
    let list = List::new(items)
    .block(Block::bordered().title("Chats"))
    .highlight_style(Style::new().reversed());

    frame.render_stateful_widget(list, chatlog_area, &mut chatlog_state.current_selection);
}






// Static historical chats functions
// not for dynamic use
/// Displays a single *static* chat message in a bordered bubble format
/// 
/// Do not use for dynamic chats
///
/// # Arguments
/// * `sender` - Name of the message sender
/// * `timestamp` - Message timestamp string
/// * `message` - The message content (will be word-wrapped)
/// * `is_left` - If true, aligns bubble to left; if false, aligns to right
/// 
/// The bubble automatically adjusts its width based on content:
/// - MIN_WIDTH (30 chars): Minimum bubble size
/// - MAX_WIDTH (50 chars): Maximum bubble size
/// - Width adapts to fit the longest content line
/// 
/// Format:
/// ```
/// ┌────────────────┐
/// │ Nov 2, 10:23 PM│
/// │ Sarah          │
/// ├────────────────┤
/// │ Hello there!   │
/// └────────────────┘
/// ```
pub fn display_chat_bubble(sender: &str, timestamp: &str, message: &str, is_left: bool) {
    const MAX_WIDTH: usize = 50;  // Maximum bubble width
    const MIN_WIDTH: usize = 30;  // Minimum bubble width
    
    // Calculate required width based on content
    let sender_len = sender.len();
    let timestamp_len = timestamp.len();
    
    // Word wrap message to MAX_WIDTH
    let wrapped_lines = word_wrap(message, MAX_WIDTH - 4);
    
    // Find the longest line to determine bubble width
    let longest_message = wrapped_lines.iter()
        .map(|line| line.len())
        .max()
        .unwrap_or(0);
    
    // Bubble width = longest content + padding
    let bubble_width = sender_len
        .max(timestamp_len)
        .max(longest_message)
        .max(MIN_WIDTH)
        .min(MAX_WIDTH) + 2; // +2 for padding
    
    let indent = if is_left { String::new() } else { " ".repeat(bubble_width) };
    
    // Top border
    println!("{}┌{}┐", indent, "─".repeat(bubble_width));
    
    // Timestamp
    println!("{}│ {:<width$} │", indent, timestamp, width = bubble_width - 2);
    
    // Sender
    println!("{}│ {:<width$} │", indent, sender, width = bubble_width - 2);
    
    // Separator
    println!("{}├{}┤", indent, "─".repeat(bubble_width));
    
    // Message (each wrapped line)
    for line in &wrapped_lines {
        println!("{}│ {:<width$} │", indent, line, width = bubble_width - 2);
    }
    
    // Bottom border
    println!("{}└{}┘", indent, "─".repeat(bubble_width));
    println!();

    // sleep so we do not print everything at once
}

/// Wraps text to fit within a maximum width while preserving words
/// 
/// # Arguments
/// * `text` - The text to wrap
/// * `max_width` - Maximum characters per line
/// 
/// # Returns
/// Vector of strings, each representing one wrapped line
/// 
/// Words are kept intact - if a word would exceed max_width,
/// it starts on a new line. Splits on whitespace.
pub fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        // If adding this word exceeds max width, start new line
        if current_line.len() + word.len() + 1 > max_width {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
            }
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    lines
}

/// Parses and displays a complete chat conversation
/// Do not use for dynamic chats
/// # Arguments
/// * `content` - Multi-line string containing the chat log
/// * `participants` - Tuple of (person1, person2) names for the conversation
/// 
/// Expected message format per line:
/// ```
/// [Nov 2, 10:23 PM] Sarah: Hello there!
/// [Nov 2, 10:24 PM] Marcus: Hi Sarah!
/// ```
/// 
/// Messages from person1 align left, messages from person2 align right.
/// Displays a centered header with participant names.
/// Sleeps 2 seconds between messages for readability.
pub fn parse_and_display_chat(content: &str, participants: (&str, &str)) {
    let (person1, person2) = participants; // e.g., ("Sarah", "Marcus")
    let mut header = String::new();
    header += "\n┌─────────────────────────────────────┐\n";
    header +="│ TEXT MESSAGE LOG                    │\n";
    header += format!("│ {} ↔ {} │\n", person1, person2).as_str();
    header +="└─────────────────────────────────────┘\n";
    header += "Hold ESC to display all at once";
    let header = terminal::center_multiline(header);
    println!("{}",header);

    let mut wait = true;
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Parse: "[Nov 2, 10:23 PM] Sarah: Message here"
        if let Some((timestamp, rest)) = parse_message_line(line) {
            if let Some((sender, message)) = rest.split_once(": ") {
                let is_left = sender.trim() == person1;
                display_chat_bubble(sender.trim(), &timestamp, message.trim(), is_left);
                if wait {thread::sleep(Duration::from_secs(2))};
            }
        }

        let key = terminal::get_input_now();
        if key.is_esc() {
            wait = false
        }
    }

    menu_components::wait_for_scroll();
}

/// Extracts timestamp and remaining content from a chat message line
/// 
/// # Arguments
/// * `line` - A line of text starting with a bracketed timestamp
/// 
/// # Returns
/// Some((timestamp, rest)) if line starts with [timestamp], None otherwise
/// 
/// Example:
/// Input: "[Nov 2, 10:23 PM] Sarah: Hello"
/// Output: Some(("Nov 2, 10:23 PM", "Sarah: Hello"))
fn parse_message_line(line: &str) -> Option<(String, String)> {
    if line.starts_with('[') {
        if let Some(end) = line.find(']') {
            let timestamp = &line[1..end];
            let rest = &line[end + 1..].trim();
            return Some((timestamp.to_string(), rest.to_string()));
        }
    }
    None
}