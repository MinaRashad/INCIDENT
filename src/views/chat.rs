use std::thread;
use std::time::Duration;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::event::poll;
use crossterm::event::read;

use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::ToText;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::BorderType;
use ratatui::widgets::Wrap;



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

        // input logic is here
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
                        // +shift means changing chat
                        if k.modifiers.contains(KeyModifiers::SHIFT)
                        {
                            sound::play(sound::SoundCategory::GUIFeedback);
                            chatApp_state.current_selection.select_next();
                        }
                        else{
                            chatApp_state.chat_scroll += 1
                        }

                    },
                    KeyCode::Up => {
                        if k.modifiers.contains(KeyModifiers::SHIFT)
                        {
                            sound::play(sound::SoundCategory::GUIFeedback);
                            chatApp_state.current_selection.select_previous();
                        }
                        else{
                            chatApp_state.chat_scroll = (chatApp_state.chat_scroll.saturating_sub(1)).max(0)
                        }
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

    let general_layout = Layout::vertical(
                        [
                            Constraint::Length(5), // header
                            Constraint::Fill(1), // app content
                            Constraint::Length(1) // help
                        ]
                    ).split(frame_area);
    
    let app_layout = Layout::horizontal(
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
    

    let help = Paragraph::new("[ESC] close  [↑↓] scroll  [SHIFT+↑↓] change chat")
                .centered().reversed();

    
    frame.render_widget(header, general_layout[0]);
    frame.render_widget(help, general_layout[2]);
    frame.render_widget(current_chat, app_layout[1]);

    render_chatlogs(frame, 
        app_layout[0],
         chat_app_state);

    let chat_log = ChatLog {
        sender: NPC::Marcus,
        messages: vec![
            Message {
                is_recieved: true,
                content: "Hey! How are you doing?".to_string(),
            },
            Message {
                is_recieved: false,
                content: "I'm good, thanks! How about you?".to_string(),
            },
            Message {
                is_recieved: true,
                content: "Pretty good! Did you finish that project we were talking about last week?".to_string(),
            },
            Message {
                is_recieved: false,
                content: "Almost done, just debugging some really annoying issues with the database connections. You know how it goes.".to_string(),
            },
            Message {
                is_recieved: true,
                content: "Oh yeah, database stuff is always a pain. What kind of errors are you getting?".to_string(),
            },
            Message {
                is_recieved: false,
                content: "Mostly timeout issues when the server gets under heavy load. I think it's a connection pool problem but I'm not entirely sure yet.".to_string(),
            },
            Message {
                is_recieved: true,
                content: "Have you tried increasing the pool size or adjusting the timeout settings?".to_string(),
            },
            Message {
                is_recieved: false,
                content: "Yeah I tried both. Increasing pool size helped a bit but didn't solve it completely. I'm thinking maybe there's a leak somewhere.".to_string(),
            },
            Message {
                is_recieved: true,
                content: "Connection leaks are tricky. Make sure you're properly closing connections in your finally blocks or using context managers if you're in Python.".to_string(),
            },
            Message {
                is_recieved: false,
                content: "Good point. I'll audit the code to make sure all connections are being properly released. Thanks for the suggestion!".to_string(),
            },
            Message {
                is_recieved: true,
                content: "No problem! Let me know if you need a second pair of eyes on the code. I've dealt with similar issues before.".to_string(),
            },
            Message {
                is_recieved: false,
                content: "Will do, I really appreciate it. How's your project going by the way?".to_string(),
            },
            Message {
                is_recieved: true,
                content: "It's going well actually. We just finished the beta testing phase and the feedback has been mostly positive. A few bugs to fix but nothing major.".to_string(),
            },
            Message {
                is_recieved: false,
                content: "That's awesome! When are you planning to launch?".to_string(),
            },
            Message {
                is_recieved: true,
                content: "Probably in about two weeks if everything goes smoothly. Still need to finalize some UI tweaks and update the documentation.".to_string(),
            },
            Message {
                is_recieved: false,
                content: "Exciting times! I'll definitely check it out when it launches.".to_string(),
            },
        ],
    };

    render_conversation(frame, app_layout[1],
         chat_log, chat_app_state);
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

fn render_conversation(frame: &mut Frame, 
    conversation_area:Rect, chat_log: ChatLog, chat_app_state:&ChatAppState){
        
    let msgs = chat_log.messages;

    let chat_bubble_width = conversation_area.width * 45 / 100;
    
    let start = (chat_app_state.chat_scroll)
                .min(msgs.len());

    
    let mut visible_message_count = 0; // how messages can we see now
    let mut constraints : Vec<Constraint> = vec![];
    let mut total_height = 0; // total height of the messages

    for msg in &msgs[start..] {
        let msg_height = calculate_message_height(&msg.content, chat_bubble_width);
        
        if total_height + msg_height > conversation_area.height {
            break;
        }
        visible_message_count += 1;
        
        constraints.push(Constraint::Length(msg_height));
        total_height += msg_height;
    }
    
    let messages_layout = Layout::vertical(constraints)
                                .split(conversation_area);

    // iterate through messages to display bubbles
    for i in (start)..(start+visible_message_count)
    {
            let msg = &msgs[i];
            let content = &msg.content;
            let is_recieved = msg.is_recieved;
            let npc = chat_log.sender;

            let message_bubble = Paragraph::new(content.to_text())
                        .centered()
                        .block(Block::bordered()
                        .title(if is_recieved{
                            npc.name()
                        }else{
                            "You"
                        })
                        .title_style(Style::new().bold())
                    )
                        .wrap(Wrap{trim:true});
            let layout_idx = i - start;
            let message_row = messages_layout[layout_idx];

            // lets split this to a horizontal layout
            let row_layout = Layout::horizontal([
                                Constraint::Fill(1),        // left padding
                                Constraint::Percentage(45), // incoming messages (left side)
                                Constraint::Fill(1),        // center padding
                                Constraint::Percentage(45), // outgoing messages (right side)
                                Constraint::Fill(1),        // right padding
                            ]).split(message_row);

            let (LEFT, RIGHT) = (1,3);
            let side = if is_recieved {
                            LEFT
                        } else {
                            RIGHT
                        };
            


            frame.render_widget(message_bubble, row_layout[side]);
        }

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

fn calculate_message_height(content: &str, width: u16) -> u16 {
    let content_width = width.saturating_sub(2) as usize; // subtract borders
    let wrapped_lines = word_wrap(content, content_width);
    wrapped_lines.len() as u16 + 2 // number of lines + 2 for borders
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