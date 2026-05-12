# INCIDENT - TUI Detective Game

## Project Overview
**INCIDENT** is a terminal-based (TUI) detective simulation game written in Rust. Players take on the role of an investigator, navigating a simulated operating system to solve cases by exploring documents, chatting with NPCs, and identifying contradictions in evidence.

### Core Technologies
- **Rust**: Language of choice for performance and safety.
- **Ratatui**: Terminal UI framework for rendering the game interface.
- **Crossterm**: Low-level terminal handling (input, terminal size, etc.).
- **SQLite (rusqlite)**: Used for the game's "all-seeing" persistent state, storing player information, document metadata, chat history, and events.
- **Rodio**: Audio playback for sound effects and ambient music.
- **Serde/Serde_JSON**: Data serialization for configuration and dialogue files.

### Key Architecture Components
1.  **GameState Machine (`src/game_state.rs`)**: Controls the primary flow between different screens (Title, Console, Documents, Chat, etc.).
2.  **Windowing System (`src/windows.rs`)**: Simulates a multi-window OS by spawning new terminal instances of the same executable with specific CLI flags (e.g., `--docs`, `--chats`).
3.  **Event Processor (`src/events.rs`)**: A background thread ("The All-Seeing Eye") that monitors the `history` table in the database and triggers game effects (e.g., clearance changes, endings) when certain conditions are met.
4.  **Dialogue System (`assets/Chat/CHAT_SCHEMA.md`)**: A node-based JSON dialogue system with condition checking and event triggering tied to the database.
5.  **Metadata-Driven Investigation**: Documents are linked to metadata tags in the database. Contradictions are identified when two documents have different values for the same tag (e.g., conflicting timestamps for an event).

---

## Building and Running

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)

### Commands
- **Run the game**: `cargo run`
- **Start a new game**: Run `_new_game.bat` (Windows) or manually delete `main.db` and run `cargo run`.
- **Open specific modules (internal use)**:
    - `cargo run -- --docs`
    - `cargo run -- --chats`
- **Build release**: `cargo build --release`

---

## Directory Structure
- `src/`: Rust source code.
    - `data/`: Database logic and data structures (player, chat, docs).
    - `views/`: UI implementation for various game screens.
    - `game_state/`: High-level game flow and endings logic.
- `assets/`: Game assets.
    - `documents/`: The "in-game" file system containing case evidence.
    - `Chat/`: Dialogue JSON files and schemas.
    - `sounds/`: Audio assets.
    - `Images/`: Visual assets (mostly placeholders/converted to ASCII).
- `default.sql`: Initial database schema and seed data for a new game.
- `_new_game.bat`: Utility script to reset progress.

---

## Development Tools
- `dev_helper/dialogue.html`: A visual, node-based editor for creating and managing NPC conversation trees. Supports visual linking, condition/event management, and JSON export/import.

---

## Development Conventions

### Data & Database
- The game state is primarily driven by `main.db`.
- When adding new content (cases, documents), entries should be added to `default.sql` to ensure they are available in new game instances.
- Use `thread_local!` database connections via `METADATA_DB` defined in `src/data.rs`.

### UI & Animation
- Follow the patterns in `src/animate.rs` and `src/views/` for consistent TUI styling.
- Use `ratatui` widgets and layouts to maintain responsiveness across different terminal sizes.

### Testing
- Manual testing is currently preferred due to the heavy TUI and sound dependency.
- Verification of new dialogue or document metadata should be done by inspecting `main.db` or running the game in the relevant mode.
