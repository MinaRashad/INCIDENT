use ratatui::widgets::ListState;


pub struct ChatLog{
    sender : NPC,
    messages : Vec<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NPC {
    Marcus,
    Sarah,
    Jessica,
    David,
    Mike,
    Rodriguez,
    Elizabeth,
    Robert,
    Susan,
    Jennifer
}

impl NPC{
    pub const ALL: [NPC;10] = [NPC::David, NPC::Elizabeth, NPC::Jennifer, NPC::Jessica, NPC::Marcus,
                NPC::Mike, NPC::Robert, NPC::Rodriguez, NPC::Sarah, NPC::Susan];
}

pub struct Message{
    is_recieved: bool,
    content:String
}

#[derive(Debug,Default)]
pub struct ChatAppState{
    pub current_selection:ListState
}
