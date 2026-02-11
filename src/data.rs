pub mod player;
pub mod chat;

#[derive(Clone, Debug)]
pub struct Person{
    name:String
}

pub struct Message{
    from: Person,
    to:Person,
    content:String
}

pub struct ChatLog{
    between: (Person, Person),
    messages: Vec<Message>
}
