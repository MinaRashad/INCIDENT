
use crate::data::Person;

use std::sync::OnceLock;
use whoami;

pub static PLAYER:OnceLock<Person> = OnceLock::new();


pub fn init_player(){
    let user = 
            whoami::realname()
            .or(whoami::username());

    let user = match user{
        Ok(name)=>name,
        Err(_)=>"You".to_string()
    };
    let user = Person {name:user};

    let _ = PLAYER.set(user);
}

pub fn get_player()->Person{
    match PLAYER.get() {
        Some(person)=>person.clone(),
        None=>panic!("The name is not initialized")
    }
}
