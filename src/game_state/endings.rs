use crate::game_state::GameState;
use crate::terminal;
use crate::menu_components;
// Ending
#[derive(Clone)]
pub enum Ending{
    DepressedEnding
}




/// Displays the game ending and exits
/// 
/// # Arguments
/// * `ending` - The ending variant to display
/// 
/// Shows the ending text, waits for user input, then exits the program
pub fn show_ending(ending: Ending) 
-> GameState
{
    terminal::clear_screen();
    terminal::clear_scrollback();
    match ending {
        Ending::DepressedEnding => {
            let title = terminal::figlet_figure("THE END".to_string());
            let title = terminal::center_multiline(title);
            let title = terminal::foreground_color(title, [100, 100, 100]);
            
            println!("{}", title);
            println!();
            println!();
            
            let text = "You turned down Marcus's offer.\n\
                        It seemed like the safe choice at the time.\n\n\
                        Three weeks later:\n\
                        'Marcus Chen found dead - Case closed as suicide'\n\n\
                        You never found out why.\n\
                        Some mysteries are never solved.";
            
            let text = terminal::center_multiline(text.to_string());
            let text = terminal::faint(text);
            
            println!("{}", text);
        }
    }
    
    println!();
    println!();
    menu_components::wait_for_input();
    
    
    GameState::Exit
}


pub fn end(){
    std::process::exit(0)
}