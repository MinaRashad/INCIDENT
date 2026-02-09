use crate::terminal;
use crate::menu_components;
use crate::GameState;
use crate::animate;
use crate::windows;

use crate::terminal::*;

use std::thread;
use std::time::Duration;

pub fn start_up()->GameState{
    // Police system boot sequence
    animate::typer("POLICE INVESTIGATION NETWORK - TERMINAL BOOT v3.2.1\n", 5, false);
    animate::typer("Department of Justice - Criminal Investigations Division\n", 5, false);
    thread::sleep(Duration::from_millis(100));
    animate::typer("Initializing secure connection...\n", 8, false);
    animate::typer("Verifying credentials...\n", 8, false);
    thread::sleep(Duration::from_millis(200));
    
    // System initialization
    animate::typer("\n[OK] Evidence Management System online\n", 5, false);
    animate::typer("[OK] Case Database connected\n", 5, false);
    animate::typer("[OK] Forensics Lab Interface ready\n", 5, false);
    animate::typer("[OK] Secure Communications established\n", 5, false);
    animate::typer("[OK] Chain of Custody Logger active\n", 5, false);
    thread::sleep(Duration::from_millis(300));
    
    terminal::clear_screen();

    // Login sequence
    animate::typer("\n═══════════════════════════════════════════════════\n", 1, false);
    animate::typer("  POLICE INVESTIGATION NETWORK ACCESS POINT\n", 1, false);
    animate::typer("  Unauthorized access is a federal crime\n", 1, false);
    animate::typer("═══════════════════════════════════════════════════\n\n", 1, false);
    
    animate::typer("BADGE NUMBER: ", 15, false);
    thread::sleep(Duration::from_millis(800));
    animate::typer("DET-4729\n", 50, true);
    animate::typer("PIN: ", 15, false);
    thread::sleep(Duration::from_millis(600));
    animate::typer("***********\n", 80, true);
    thread::sleep(Duration::from_millis(500));
    
    animate::typer("\nAuthenticating...", 15, false);
    thread::sleep(Duration::from_millis(800));
    animate::typer(" VERIFIED\n", 10, false);
    thread::sleep(Duration::from_millis(400));
    
    terminal::clear_screen();

    animate::typer("ACCESS GRANTED\n", 8, false);
    animate::typer("Last login: Tue Feb  4 14:32:11 2025 from Precinct 12\n", 8, false);
    thread::sleep(Duration::from_millis(300));
    
    // Department header
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!(  "║     HOMICIDE INVESTIGATION DATABASE - COLD CASE UNIT       ║");
    println!(  "║          Classified - Law Enforcement Access Only          ║");
    println!(  "╚════════════════════════════════════════════════════════════╝\n");
    
    thread::sleep(Duration::from_millis(400));
    
    animate::typer("DET-4729@HOMICIDE-DB:~$ ", 20, false);
    thread::sleep(Duration::from_millis(600));
    animate::typer("open-case --id INCIDENT-7D3\n", 40, true);
    thread::sleep(Duration::from_millis(400));

    // Case file display
    animate::typer("\n════════════════════════════════════════════════════\n", 8, false);
    animate::typer("CASE FILE: INCIDENT-7D3\n", 10, false);
    animate::typer("════════════════════════════════════════════════════\n", 8, false);
    animate::typer("STATUS    : COLD CASE - UNSOLVED\n", 10, false);
    animate::typer("REPORTED  : 2024-08-15 03:47:22 UTC\n", 10, false);
    animate::typer("LOCATION  : Luxury Residential Compound, Unit 1847\n", 10, false);
    animate::typer("VICTIM    : Male, 41 years old\n", 10, false);
    animate::typer("════════════════════════════════════════════════════\n\n", 8, false);
    
    animate::typer("CASE SUMMARY:\n", 10, false);
    animate::typer("- Cause of Death: Gunshot wound, posterior cranium\n", 8, false);
    animate::typer("- Point of Entry: Unknown (No signs of forced entry)\n", 8, false);
    animate::typer("- Security Footage: 47-minute gap during TOD window\n", 8, false);
    animate::typer("- Scene: All doors/windows locked from interior\n", 8, false);
    animate::typer("- Ruling: Homicide \n\n", 8, false);
    
    thread::sleep(Duration::from_millis(400));
    
    animate::typer("EVIDENCE LOCKER:\n", 10, false);
    animate::typer("  [1] Digital Communications - Victim's message history\n", 8, false);
    animate::typer("  [2] Security Recordings - Building surveillance system\n", 8, false);
    animate::typer("  [3] Digital Forensics - Recovered file system image\n", 8, false);
    animate::typer("  [4] Medical Examiner - Complete autopsy report\n", 8, false);
    animate::typer("  [5] Witness Statements - Building residents & staff\n\n", 8, false);
    
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