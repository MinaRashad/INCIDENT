
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{self};
use crossterm::event::KeyCode;
use rascii_art;

use crate::data::{self, ImageDoc};
use crate::terminal;
use crate::sound;
use crate::GameState;

pub fn check_password(real_password:&str, 
        from:GameState, to:GameState)->GameState{
    
    println!("Please enter your password: ");

    let mut password:String = String::new();
    let mut attempts: i32 = 2;

    while attempts > 3 {
        io::stdin()
        .read_line(&mut password)
        .expect("Failed");
        if password.trim() != real_password {
            attempts -= 1;
            println!("{}", 
                terminal::foreground_color(format!("incorrect password ({attempts} REMAINING)"),
                 [200,100,100]));
            password = String::new(); // remove current data
        }
        else{
            println!("{}", 
                terminal::foreground_color("ACCESS GRANTED".to_string(), 
                 [100,200,100]));

            return from;
        }
    };
    return to;
}



// complex menus

fn display_options(options:Vec<String>, centered:bool) -> Vec<String>{
    for i in 0..options.len(){
        let option = &options[i];
        let option = option.to_string();
        let option = if centered{terminal::center(option)} 
                     else {option};
        println!("{option}");
    }

    // flush everything
    io::stdout().flush()
            .expect("Failed to flush");

    options
}

fn highlight_option(option:String, num_options:usize,
                    curr_selection:usize,
                    centered:bool){
        // move cursor up option.len - curr_selection
        terminal::move_cursor_up(num_options - curr_selection);
        terminal::move_cursor_linestart();

        //center first
        let option = if centered{terminal::center(option)} 
                             else {option};

        // first we need to print the inverted text at the current selection
        let selection = terminal::invert(option);
        print!("{}",selection);

        // flush the output:
        io::stdout().flush()
            .expect("Failed to flush");
        
        // move cursor down curr_selection 
        terminal::move_cursor_down( num_options - curr_selection);
        
}

fn unhighlight_option(option:String, 
                        num_options:usize, 
                        curr_selection:usize,
                        centered:bool){
        // move cursor up option.len - curr_selection
        terminal::move_cursor_up(num_options - curr_selection);
        terminal::move_cursor_linestart();

        //center first
        let option = if centered{terminal::center(option)} 
                             else {option};
        
        print!("{}",&option);

        // flush the output:
        io::stdout().flush()
            .expect("Failed to flush");
        
        // move cursor down curr_selection 
        terminal::move_cursor_down( num_options - curr_selection);
        
}


pub fn multichoice(title:&str, options:Vec<GameState>,
                    centered:bool)-> GameState{

    
    terminal::hide_cursor();
    // handle unexpected cases
    if options.is_empty() {panic!("There are no options")};

    let options_str: Vec<String> = options.iter()
                .map(|state| state.as_name())
                .collect();

    // print the title
    let title = if centered {terminal::center(title.to_string())} 
                        else{title.to_string()};
    let title = terminal::bold(title);
    let help = "Select your choice (Use ↕ and ↩)".to_string();
    let help = if centered{ terminal::center(help)}
                       else{help};
    let help = terminal::blink(help);
    
    println!("{}",title);
    println!("{}", help);

    let options_str =
        options_str.iter().map(
                |s| s.to_string()
            )
            .collect();

    let options_str = display_options(options_str, centered);
    

    let mut curr_selection :usize= 0;

    // input buffer
    let input_buffer = time::Duration::from_millis(200);
    let mut now = time::SystemTime::now();

    loop{
        highlight_option(options_str[curr_selection].to_string(),
                        options_str.len(),
                        curr_selection,
                        centered);

        let elapsed = now.elapsed().expect("Getting elapsed time failed");
        if elapsed < input_buffer {
            sleep(input_buffer - elapsed);
        }

        // now we can wait for an input
        let input = terminal::get_input();
        
        if input.is_down() || input.is_up(){

            unhighlight_option(options_str[curr_selection].to_string(),
                        options_str.len(),
                        curr_selection,
                        centered);

            now = time::SystemTime::now();

            sound::play(sound::SoundCategory::GUIFeedback);

        }
        if input.is_down() {
            curr_selection = (curr_selection + 1) % options.len();
        } else if input.is_up() {
            curr_selection =  if curr_selection == 0 {curr_selection + options.len()} else {curr_selection};
            curr_selection -= 1;
        } else if input.is_enter() {
            break;
        }

    }

    terminal::show_cursor();
    options[curr_selection].clone()
}


// graphical functions
pub fn display_image(img:ImageDoc, w:Option<u32>, h:Option<u32>)-> Option<String>{

    let mut image_ascii = String::new();
    let options = rascii_art::RenderOptions::new();
    let options = options.colored(true);
    let mut options = options.charset(rascii_art::charsets::BLOCK);

    if let Some(w) = w{
        options = options.width(w);
    }
    
    if let Some(h) = h {
        options = options.height(h);
    }

    let path = match img {
        ImageDoc::Image(path)=>path,
        ImageDoc::EncyptedImage(path, _ ) => panic!("Cant decrypt, please only pass plain images")
    };

    let path = path.to_str()?;

    let result = rascii_art::render_to(path, &mut image_ascii, 
        &options
        );
    
    if result.is_ok(){
        return Some(image_ascii);
    }

    return None
}

pub fn print_logo(){
    let [w, h] = terminal::size();
    let w = w as u32;
    let h= h as u32;

    let w = (70*w)/100;
    let logo_path = data::docs::OS_LOGO_PATH;
    let logo_path = PathBuf::from(logo_path);
    let logo = ImageDoc::Image(logo_path);
    
    if let Some(img) = display_image(logo, Some(w), None)
    {
        //let img = terminal::center_multiline(img);
        terminal::move_cursor_linestart();
        let img = terminal::center_multiline(img);
        println!("{}",img)
    }
}

// utilities used inside views
pub fn date()->Result<[u64; 3],time::SystemTimeError>
{
    

    // let date = format!("{day}-{month}-2230",day=day-sum, month=month);
    let now = time::SystemTime::now();
    // epoch is 1-1-1970
    let interval = now.duration_since(time::UNIX_EPOCH)?;

    let interval = interval.as_secs();

    // now we have how many seconds passed
    // a year on average has 356 day (unless divisible by 4)
    let minutes = interval/60;
    let hours = minutes/60;
    let mut days = hours/24;

    let mut curr_year = 1970;
    let month_days = [31,28,31,30,31,30,31,31,30,31,30,31];
    
    // finding current year
    while days > 365{
        // account for leap day
        if curr_year % 4 == 0{
            if curr_year % 100 == 0 && curr_year % 400 == 0{
                days -= 1 // leap day
            }
            else if curr_year % 100 == 0 && curr_year % 400 != 0{
                // nothing happens
            }
            else{
                days -= 1
            }
        }

        for month in month_days{
            days -= month as u64;
        }
        curr_year += 1
    }
    let mut curr_month  = 0;

    for month in month_days{
        curr_month += 1;

        days -= month as u64;
        if days <= 31{
            break
        }
    }

     

    Ok([curr_year,curr_month,days])
}

pub fn wait_for_input(){
    let sub = "Press anything to Continue...".to_string();
    let sub = terminal::center(sub);
    let sub = terminal::blink(sub);
    println!("{}",sub);

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("An error occured");
}

pub fn wait_for_scroll(){
    let sub = "Mouse wheel scroll. Enter to exit".to_string();
    let sub = terminal::center(sub);
    let sub = terminal::blink(sub);
    println!("{}",sub);


    loop {
        let input: KeyCode = terminal::get_input();

        match input {
            KeyCode::Enter => break,
            _ => {}
        }
    }
}


