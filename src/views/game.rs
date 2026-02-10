use crate::terminal;
use crate::menu_components;
use crate::GameState;
use crate::animate;
use crate::windows;
use crate::sound;

use std::thread;
use std::time::Duration;

pub fn start_up()->GameState{

    sound::boot_play();
    // Police system boot sequence
    let boot_header = terminal::foreground_color(
        "POLICE INVESTIGATION NETWORK - TERMINAL BOOT v3.2.1".to_string(),
        [0, 255, 255] // Cyan
    );
    animate::typer(&format!("{}\n", boot_header), 5, false);
    
    let dept_header = terminal::foreground_color(
        "Department of Justice - Criminal Investigations Division".to_string(),
        [100, 200, 255] // Light blue
    );
    animate::typer(&format!("{}\n", dept_header), 5, false);
    thread::sleep(Duration::from_millis(100));
    
    animate::typer("Initializing secure connection...\n", 8, false);
    animate::typer("Verifying credentials...\n", 8, false);
    thread::sleep(Duration::from_millis(200));
    
    // System initialization - Green for OK status
    let ok_msg = |text: &str| {
        terminal::foreground_color(
            format!("[OK] {}", text),
            [0, 255, 0] // Green
        )
    };
    
    animate::typer(&format!("{}\n", ok_msg("Evidence Management System online")), 5, false);
    animate::typer(&format!("{}\n", ok_msg("Case Database connected")), 5, false);
    animate::typer(&format!("{}\n", ok_msg("Forensics Lab Interface ready")), 5, false);
    animate::typer(&format!("{}\n", ok_msg("Secure Communications established")), 5, false);
    animate::typer(&format!("{}\n", ok_msg("Chain of Custody Logger active")), 5, false);
    thread::sleep(Duration::from_millis(300));
    
    terminal::clear_screen();

    // Login sequence - Yellow/Orange for warnings
    let separator = terminal::foreground_color(
        "═══════════════════════════════════════════════════".to_string(),
        [255, 200, 0] // Gold
    );
    animate::typer(&format!("{}\n", separator), 1, false);
    
    let title = terminal::bold(
        terminal::foreground_color(
            "  POLICE INVESTIGATION NETWORK ACCESS POINT".to_string(),
            [255, 255, 255] // White
        )
    );
    animate::typer(&format!("{}\n", title), 1, false);
    
    let warning = terminal::foreground_color(
        "  Unauthorized access is a federal crime".to_string(),
        [255, 100, 0] // Orange/Red
    );
    animate::typer(&format!("{}\n", warning), 1, false);
    animate::typer(&format!("{}\n\n", separator), 1, false);
    
    let badge_prompt = terminal::foreground_color("BADGE NUMBER: ".to_string(), [200, 200, 200]);
    animate::typer(&badge_prompt, 15, false);
    thread::sleep(Duration::from_millis(800));
    animate::typer("DET-4729\n", 50, true);
    
    let pin_prompt = terminal::foreground_color("PIN: ".to_string(), [200, 200, 200]);
    animate::typer(&pin_prompt, 15, false);
    thread::sleep(Duration::from_millis(600));
    animate::typer("***********\n", 80, true);
    thread::sleep(Duration::from_millis(500));
    
    animate::typer("\nAuthenticating...", 15, false);
    thread::sleep(Duration::from_millis(800));
    let verified = terminal::bold(
        terminal::foreground_color(" VERIFIED".to_string(), [0, 255, 0])
    );
    animate::typer(&format!("{}\n", verified), 10, false);
    thread::sleep(Duration::from_millis(400));
    
    terminal::clear_screen();

    let access_granted = terminal::bold(
        terminal::foreground_color("ACCESS GRANTED".to_string(), [0, 255, 0])
    );
    animate::typer(&format!("{}\n", access_granted), 8, false);
    animate::typer("Last login: Tue Feb  4 14:32:11 2025 from Precinct 12\n", 8, false);
    thread::sleep(Duration::from_millis(300));
    
    // Department header - Inverted style for emphasis
    let header_line1 = terminal::foreground_color(
        "╔════════════════════════════════════════════════════════════╗".to_string(),
        [0, 200, 255]
    );
    let header_line2 = terminal::bold(
        terminal::foreground_color(
            "║     HOMICIDE INVESTIGATION DATABASE - COLD CASE UNIT       ║".to_string(),
            [0, 200, 255]
        )
    );
    let header_line3 = terminal::foreground_color(
        "║          Classified - Law Enforcement Access Only          ║".to_string(),
        [255, 100, 100] // Light red for classified
    );
    let header_line4 = terminal::foreground_color(
        "╚════════════════════════════════════════════════════════════╝".to_string(),
        [0, 200, 255]
    );
    
    println!("\n{}", header_line1);
    println!("{}", header_line2);
    println!("{}", header_line3);
    println!("{}\n", header_line4);
    
    thread::sleep(Duration::from_millis(400));
    
    let prompt = terminal::foreground_color("DET-4729@HOMICIDE-DB:~$ ".to_string(), [0, 255, 0]);
    animate::typer(&prompt, 20, false);
    thread::sleep(Duration::from_millis(600));
    animate::typer("open-case --id INCIDENT-7D3\n", 40, true);
    thread::sleep(Duration::from_millis(400));

    // Case file display
    let case_separator = terminal::foreground_color(
        "════════════════════════════════════════════════════".to_string(),
        [255, 255, 0] // Yellow
    );
    animate::typer(&format!("{}\n", case_separator), 8, false);
    
    let case_title = terminal::bold(
        terminal::foreground_color("CASE FILE: INCIDENT-7D3".to_string(), [255, 255, 255])
    );
    animate::typer(&format!("{}\n", case_title), 10, false);
    animate::typer(&format!("{}\n", case_separator), 8, false);
    
    let status = terminal::foreground_color(
        "STATUS    : ".to_string(),
        [200, 200, 200]
    ) + &terminal::foreground_color("COLD CASE - UNSOLVED".to_string(), [255, 100, 100]);
    animate::typer(&format!("{}\n", status), 10, false);
    
    if let Ok(date) = menu_components::date(){
        let reported_date = format!(
            "REPORTED  : {0}-{1}-{2} 03:47:22 UTC\n"
                ,date[0], date[1], date[2]);
        animate::typer(&reported_date, 10, false);
    };
    animate::typer("LOCATION  : Luxury Residential Compound, Unit 1847\n", 10, false);
    animate::typer("VICTIM    : Male, 41 years old\n", 10, false);
    animate::typer(&format!("{}\n\n", case_separator), 8, false);
    
    let summary_header = terminal::bold(
        terminal::foreground_color("CASE SUMMARY:".to_string(), [255, 200, 0])
    );
    animate::typer(&format!("{}\n", summary_header), 10, false);
    animate::typer("- Cause of Death: Gunshot wound, posterior cranium\n", 8, false);
    animate::typer("- Point of Entry: Unknown (No signs of forced entry)\n", 8, false);
    animate::typer("- Security Footage: 47-minute gap during TOD window\n", 8, false);
    animate::typer("- Scene: All doors/windows locked from interior\n", 8, false);
    
    let ruling = terminal::foreground_color("- Ruling: ".to_string(), [200, 200, 200]) 
        + &terminal::bold(terminal::foreground_color("Homicide".to_string(), [255, 0, 0]));
    animate::typer(&format!("{}\n\n", ruling), 8, false);
    
    thread::sleep(Duration::from_millis(400));
    
    let evidence_header = terminal::bold(
        terminal::foreground_color("EVIDENCE LOCKER:".to_string(), [255, 200, 0])
    );
    animate::typer(&format!("{}\n", evidence_header), 10, false);
    
    let evidence_item = |num: &str, text: &str| {
        format!("  {} {}\n",
            terminal::foreground_color(format!("[{}]", num), [0, 255, 255]),
            text
        )
    };
    
    animate::typer(&evidence_item("1", "Digital Communications - Victim's message history"), 8, false);
    animate::typer(&evidence_item("2", "Security Recordings - Building surveillance system"), 8, false);
    animate::typer(&evidence_item("3", "Digital Forensics - Recovered file system image"), 8, false);
    animate::typer(&evidence_item("4", "Medical Examiner - Complete autopsy report"), 8, false);
    animate::typer(&evidence_item("5", "Witness Statements - Building residents & staff\n"), 8, false);
    
    thread::sleep(Duration::from_millis(300));
    
    menu_components::wait_for_input();
    GameState::MainConsole
}

pub fn main_console()->GameState{
    terminal::clear_screen();
    let selection = menu_components::multichoice(
                        "Linked resources available",
                        vec!["[CHAT LOGS]",
                                "[APARTMENT FILE SYSTEM]",
                                "[RESIDENT DIRECTORY]"], 
                        true);
    
    match selection {
        0 => windows::start_mode("chat"),
        1 => windows::start_mode("docs"),
        _ => todo!("not here yet")
    }
    
    GameState::MainConsole
}