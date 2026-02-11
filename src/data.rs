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

use std::path::{Path, PathBuf};
pub struct Password{
    content:String
}

pub enum TextDoc {
    TextDoc,
    EncryptedDoc(Password)
}

pub enum ImageDoc{
    Image(PathBuf),
    EncyptedImage(PathBuf,Password)
}


pub enum File{
    Text(TextDoc),
    Image(ImageDoc)
}

pub enum Entry{
    File(File),
    Folder(Folder)
}


pub struct Folder{
    name:String,
    children:Vec<Entry>
}
