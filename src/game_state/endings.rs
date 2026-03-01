use crate::events::EventType;
use crate::events::add_event;
use crate::game_state::GameState;
use crate::sound;
use crate::sound::SoundCategory;
use crate::terminal;
use crate::menu_components;
use crate::data;
// Ending
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Ending{
    Refusal
}

impl Ending {
    pub fn to_str(&self) -> &str {
        match self {
            Ending::Refusal => "Refusal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Refused" => Some(Ending::Refusal),
            _ => None,
        }
    }
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
     let title = terminal::figlet_figure("THE END".to_string());
    let title = terminal::center_multiline(title);
    let title = terminal::foreground_color(title, [100, 100, 100]);
    
    println!("{}", title);
    println!();
    println!();
    let text = match ending {
        Ending::Refusal => {
            sound::play_forever(SoundCategory::Sad);
            let text = "You turned down Marcus's offer.\n\
                        You wondered many times what your life\n\
                        will look like had you accepted the offer\n\
                        However, regretting the past does not matter\n\
                        All we have is only the present.";
            
            let text = terminal::center_multiline(text.to_string());
            terminal::faint(text)
            
        }
    };
    
    println!("{}", text);
    println!();
    println!();

    let message: String = format!("You got the {} ending!", ending.to_str());
    let message = terminal::center(message);
    println!("{}",message);
    println!();

    menu_components::wait_for_input();
    
    GameState::Exit
}


pub fn end(ending:Ending){

    add_event(EventType::EndGame(ending).to_str());
    std::process::exit(0)
}

/// A function that checks if the game ended
/// if it did, returns an ending, otherwise None
pub fn get_ending()->Option<Ending>{
    let result  = data::METADATA_DB.with(
        |db| 
        -> Result<EventType, rusqlite::Error>{
        let conn = db.get().expect("Database not initialized");

        let mut statement = conn
            .prepare("SELECT name 
            FROM
            history 
            WHERE
            name LIKE '_end%'
            LIMIT 1")
            .expect("Unable to create SQL statement");
        
        let ending :EventType =
            statement.query_one([],
            |row|    
            Ok(EventType::from_str(row.get::<usize,String>(0)?.as_str()))
        )?;

        Ok(ending) 
    });

    match result {
        Ok(EventType::EndGame(ending))=> Some(ending),
        _ => None
    }
}
