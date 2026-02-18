use ratatui::widgets::ListState;


pub struct ChatLog{
    pub sender : NPC,
    pub messages : Vec<Message>,
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

    pub fn name(&self) -> &str {
        match self {
            NPC::Marcus => "Marcus",
            NPC::Sarah => "Sarah",
            NPC::Jessica => "Jessica",
            NPC::David => "David",
            NPC::Mike => "Mike",
            NPC::Rodriguez => "Rodriguez",
            NPC::Elizabeth => "Elizabeth",
            NPC::Robert => "Robert",
            NPC::Susan => "Susan",
            NPC::Jennifer => "Jennifer",
        }
    }
}

pub struct Message{
    pub is_recieved: bool,
    pub content:String
}

#[derive(Debug,Default)]
pub struct ChatAppState{
    pub current_selection:ListState,
    pub chat_scroll:usize
}
