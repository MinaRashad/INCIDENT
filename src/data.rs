pub mod player;
pub mod chat;
pub mod docs;

/*
    Here I will define the model of our data
    This is similar to 3N standards
*/

// chatlogs

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

// Documents

use std::path::PathBuf;
pub struct Password{
    content:String
}

pub struct ImageDoc(pub PathBuf);

impl ImageDoc {
    // If you were creating them like this before: ImageDoc::Image(path)
    // You can keep doing that by adding a constructor:
    pub fn image(path: PathBuf) -> Self {
        ImageDoc(path)
    }

    // If you were matching on them before:
    // match doc { ImageDoc::Image(p) => ... }
    // You can update the match to look like this:
    pub fn get_path(&self) -> &PathBuf {
        &self.0
    }
}