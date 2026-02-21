use std::{thread, time::Duration};

use crossterm::event::{
    poll, read, Event, KeyCode, KeyEvent, KeyModifiers,
};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::ToText,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
};

use std::sync::OnceLock;



use crate::data::{self, chat::{self, Choice, DialogueNode}};
use crate::menu_components;
use crate::GameState;
use crate::sound;
use crate::terminal;

use crate::data::chat::{ChatAppState, NPC, ChatLog, Message};


// load all chats in this thread
static CHATS:OnceLock<data::chat::DialogueTree> = OnceLock::new();

/// Entry point for the chat log viewer
/// Waits for user input before returning to the Chats state
/// 
/// # Returns
/// Always returns GameState::Chats
pub fn start()->GameState{

    init_chats();
    let mut terminal = ratatui::init();
    
    let mut chat_app_state: ChatAppState = ChatAppState::default();
    chat_app_state.current_chat_selection.select(Some(0));
    chat_app_state.current_choice_selection = 0;
    chat_app_state.running = true;

    update_chat(&mut chat_app_state);

    println!("{:?}",chat_app_state.chatlog.messages);
    menu_components::wait_for_input();
    

    while chat_app_state.running {
        // check new messages
        update_chat(&mut chat_app_state);

        // render chat
        terminal.draw(|f| 
            render_chat(f, &mut chat_app_state))
            .expect("failed to draw a frame");

        // input logic is here
        if poll(Duration::from_millis(500)).unwrap_or(false){
            // input goes here
            match read() {
                Ok(Event::Key(k))
                if k.is_release() => handle_key_input(k, &mut chat_app_state),            
                _ => {} // do nothing if anything else including error
            };

        }

        update_chat(&mut chat_app_state);

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

    let conversation_layout = Layout::vertical(
                        [
                            Constraint::Fill(1), // chat area
                            Constraint::Length(3) // Choices
                    ]).split(app_layout[1]);

    let header = Paragraph::new("Whats'agram")
                .centered()
                .block(
                    Block::default().border_type(BorderType::Double)
                    .borders(Borders::ALL)
                );

    

    let help = Paragraph::new("[ESC] close  [↑↓] scroll  [←→] switch choice  [Enter] select  [SHIFT+↑↓] change chat")
                .centered().reversed();

    
    frame.render_widget(header, general_layout[0]);
    frame.render_widget(help, general_layout[2]);

    render_chatlogs(frame, 
        app_layout[0],
         chat_app_state);



    render_conversation(frame, conversation_layout[0],
        chat_app_state);

    render_choices(frame, conversation_layout[1], chat_app_state);
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

    frame.render_stateful_widget(list, chatlog_area, &mut chatlog_state.current_chat_selection);
}

fn render_conversation(frame: &mut Frame, 
    conversation_area:Rect, chat_app_state:&ChatAppState){
        
    let msgs = &chat_app_state.chatlog.messages;

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
    let npc = chat_app_state.chatlog.sender;

    for i in (start)..(start+visible_message_count)
    {
            let msg = &msgs[i];
            let content = &msg.content;
            let is_recieved = msg.is_recieved;

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

            let (left, right) = (1,3);
            let side = if is_recieved {
                            left
                        } else {
                            right
                        };
            


            frame.render_widget(message_bubble, row_layout[side]);
        }

}

fn render_choices(frame: &mut Frame, choice_area:Rect, 
    chat_app_state:&ChatAppState){

        let choices = &chat_app_state.choices;
        let constraints: Vec<Constraint> = choices.iter().
                    map(|_| Constraint::Fill(1))
                    .collect();

        let button_layout = Layout::horizontal(constraints)
                    .split(choice_area);

        let choices_items:Vec<Paragraph> = choices.iter()
                    .map(|choice| choice.text.clone())
                    .map(|choice| Paragraph::new(choice)
                            .block(Block::bordered()) 
                    )
                    .enumerate()
                    .map(|(i, choice)|
                        if i == chat_app_state.current_choice_selection {
                            choice.style(Style::new().reversed())
                        }
                        else{
                            choice
                        }
                    )
                    .collect();
        for (i, choice) in choices_items.iter().enumerate(){
            frame.render_widget(choice, button_layout[i]);
        }

    }

fn handle_key_input(k: KeyEvent, chat_app_state:&mut ChatAppState){
    sound::play(sound::SoundCategory::GUIFeedback);
    match k.code {
    KeyCode::Esc => chat_app_state.running=false,
    KeyCode::Down => {
        // +shift means changing chat
        if k.modifiers.contains(KeyModifiers::SHIFT)
        {
            sound::play(sound::SoundCategory::GUIFeedback);
            chat_app_state.current_chat_selection.select_next();
            chat_app_state.current_choice_selection = 0; // reset choice selection
            chat_app_state.chat_scroll = 0; // reset scroll

        }
        else{
            chat_app_state.chat_scroll += 1
        }

    },
    KeyCode::Up => {
        if k.modifiers.contains(KeyModifiers::SHIFT)
        {
            sound::play(sound::SoundCategory::GUIFeedback);
            chat_app_state.current_chat_selection.select_previous();
            chat_app_state.current_choice_selection = 0; // reset choice selection
            chat_app_state.chat_scroll = 0; // reset scroll

        }
        else{
            chat_app_state.chat_scroll = (chat_app_state.chat_scroll.saturating_sub(1)).max(0)
        }
    },
    KeyCode::Left => {
        if chat_app_state.choices.is_empty(){
            return;
        }
        let num_choices: usize = chat_app_state.choices.len();

        chat_app_state.current_choice_selection = 
                if chat_app_state.current_choice_selection == 0
                {
                    num_choices - 1
                } else {
                    chat_app_state
                            .current_choice_selection.saturating_sub(1)
                }
    },
    KeyCode::Right => {
        if chat_app_state.choices.is_empty(){
            return;
        }
        let num_choices: usize = chat_app_state.choices.len();
        chat_app_state.current_choice_selection += 1;
        chat_app_state.current_choice_selection = chat_app_state.current_choice_selection % num_choices;
    },
    KeyCode::Enter =>{
        if chat_app_state.choices.is_empty(){
            return;
        }
        let npc_idx = chat_app_state.current_chat_selection
                    .selected()
                    .unwrap_or(0);
        let npc = NPC::ALL[npc_idx];
        // we handle the choice selection
        let choice_idx = chat_app_state.current_choice_selection;
        
        let choice = &chat_app_state.choices[choice_idx];
        let mut dialogue_node = choice.to_dialogue_node();

        data::chat::process_dialogue_node(&mut dialogue_node, 
            chat::ContactChar::Player, 
            chat::ContactChar::NPC(npc));
    }
    _ => {}
    }
}

fn update_chat(chat_app_state:&mut ChatAppState)
-> Option<()>
{
    // get current selected npc
    let npcs = NPC::ALL;
    let npc_idx = chat_app_state.current_chat_selection.selected()?;
    let npc = npcs[npc_idx];

    // get all messages
    let messages = data::chat::get_messages(0, npc);
    chat_app_state.chatlog = ChatLog{sender:npc, messages};

    // we also need the current node
    let map = CHATS.get()?;
    let node = data::chat::get_current_dialogue_node(npc, map)?;

    chat_app_state.choices = node.options.clone();
    

    Some(())
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
pub fn parse_and_display_chat(content: &str) {
    let mut lines = content.lines();
        let line = lines.next();
        let line = match line {
            Some(line)=>line,
            None => return ()
        };
        let split = line.split("][");
        let split : Vec<&str> = split.into_iter().collect();
        let sender = split[1];
        let reciever = split[2];
        let reciever = match reciever.strip_suffix(']') {
            Some(r)=> r,
            None => reciever
        };
    let (person1, person2) = (sender, reciever); // e.g., ("Sarah", "Marcus")
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
        if 
        let Some((timestamp, rest)) = parse_message_line(line)
        && 
        let Some((sender, message)) = rest.split_once(": ") 
        {
            let is_left = sender.trim() == person1;
            display_chat_bubble(sender.trim(), &timestamp, message.trim(), is_left);
            if wait {thread::sleep(Duration::from_secs(2))};
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

fn init_chats()->Option<()>{
    let map: data::chat::DialogueTree = data::chat::read_dialogue_data().ok()?;
    CHATS.set(map).ok()
}