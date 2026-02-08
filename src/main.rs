mod terminal;
mod menu_components;
mod views;
mod animate;
mod windows;

use std::env;

struct Clean_up;

impl Drop for Clean_up {
    fn drop(&mut self) {
        terminal::exit_alternative_buffer();
    }
}

fn main() {
    let _ = Clean_up;

    terminal::enter_alternative_buffer();

    let args :Vec<String>= env::args().collect();


    if args.len() > 1 {
        if args[1] == "--chat".to_string(){
            views::chat();
        }
    }
    else{
        views::title_page();
        views::main_menu();
    }


}




