use whoami;

use crate::terminal;
use crate::menu_components;
use crate::GameState;

struct Person{
    name:String
}

struct Message{
    from: Person,
    to:Person,
    content:String
}

struct ChatLog{
    between: (Person, Person),
    messages: Vec<Message>
}



pub fn start()->GameState{
    println!("Chat is working");
    let user = 
            whoami::realname()
            .or(whoami::username());

    let user = match user{
        Ok(name)=>name,
        Err(_)=>"You".to_string()
    };
    

    println!("Hello {user}");


    menu_components::wait_for_input();

    GameState::Chats
}