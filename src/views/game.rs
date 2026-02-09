use crate::terminal;
use crate::menu_components;
use crate::GameState;
use crate::animate;


pub fn start_up()->GameState{
    animate::typer("INCIDENT SYSTEM v0.9.3\n", 10);
    animate::typer("Initializing secure environment...\n", 15);
    animate::typer("Integrity check ........ OK\n", 10);
    animate::typer("Memory allocation ...... OK\n", 10);
    animate::typer("Subsystems ............. ONLINE\n\n", 10);

    animate::typer("Mounting data volumes...\n", 15);
    animate::typer("/dev/casefiles ........ MOUNTED\n", 10);
    animate::typer("/dev/comms ............ MOUNTED\n", 10);
    animate::typer("/dev/archive .......... MOUNTED\n\n", 10);

    animate::typer("Loading investigation workspace...\n", 15);
    animate::typer("Spawning terminal instances [1/4]\n", 10);
    animate::typer("Spawning terminal instances [2/4]\n", 10);
    animate::typer("Spawning terminal instances [3/4]\n", 10);
    animate::typer("Spawning terminal instances [4/4]\n\n", 10);

    animate::typer("WARNING:\n", 20);
    animate::typer("Case flagged as UNRESOLVED\n", 15);
    animate::typer("Restricted material detected\n\n", 15);

    animate::typer("----------------------------------------\n", 5);
    animate::typer("CASE ID: INCIDENT-7D3\n", 10);
    animate::typer("STATUS : COLD\n", 10);
    animate::typer("LOCATION: LUXURY RESIDENTIAL COMPOUND\n", 10);
    animate::typer("SUBJECT : MALE, 41\n\n", 10);

    animate::typer("SUMMARY:\n", 10);
    animate::typer("Victim found deceased in private apartment.\n", 10);
    animate::typer("Cause of death: Ballistic trauma.\n", 10);
    animate::typer("Entry wound located posterior cranial region.\n\n", 10);

    animate::typer("NOTES:\n", 10);
    animate::typer("• No camera footage of entry or exit\n", 10);
    animate::typer("• No forced entry detected\n", 10);
    animate::typer("• Windows intact\n", 10);
    animate::typer("• Suicide ruled out\n\n", 10);

    animate::typer("----------------------------------------\n\n", 5);

    animate::typer("Linked resources available:\n", 10);
    animate::typer("[CHAT LOGS]\n", 10);
    animate::typer("[SECURITY FOOTAGE]\n", 10);
    animate::typer("[APARTMENT FILE SYSTEM]\n", 10);
    animate::typer("[AUTOPSY REPORT]\n", 10);
    animate::typer("[RESIDENT DIRECTORY]\n\n", 10);

    animate::typer("Awaiting operator input...\n", 15);


    GameState::MainMenu
}