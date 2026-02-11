use std::path::PathBuf;


use crate::menu_components;
use crate::GameState;


struct Password{
    content:String
}

enum DocType {
    TextDoc(PathBuf),
    Image(PathBuf),
    EncryptedDoc(PathBuf, Password),
    EncyptedImage(PathBuf, Password)
}

impl DocType{
    fn render(self)->String{
        todo!("Add render");
    }
}


pub fn start()->GameState{
    println!("Docs is working");
    menu_components::wait_for_input();

    return GameState::Docs;
}