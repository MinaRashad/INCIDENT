PRAGMA journal_mode=WAL;
CREATE TABLE IF NOT EXISTS metadata (
    path TEXT PRIMARY KEY,
    access_level INTEGER,
    password TEXT,
    opened INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS metadata_tags (
    path TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    value TEXT,
    PRIMARY KEY (path, tag_id),
    FOREIGN KEY (path) REFERENCES metadata(path) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS player (
    id INTEGER PRIMARY KEY CHECK (id = 0), -- Ensure only one row exists
    name TEXT NOT NULL,
    access_level INTEGER NOT NULL DEFAULT 2,
    hired INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sender      TEXT NOT NULL,
    receiver    TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE npc_dialogue_state (
    npc_name TEXT PRIMARY KEY,
    node TEXT NOT NULL,
    status   TEXT NOT NULL DEFAULT 'not_processed'
);

CREATE TABLE history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    processed_at DATETIME DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    doc_path TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS contradictions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    doc_path_a TEXT NOT NULL,
    doc_path_b TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (doc_path_a) REFERENCES metadata(path) ON DELETE CASCADE,
    FOREIGN KEY (doc_path_b) REFERENCES metadata(path) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS user_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    doc_id INTEGER NOT NULL,
    doc_table TEXT NOT NULL -- 'notes' or 'contradictions'
);

-- add starting event to history
INSERT INTO history (name) VALUES ('start');


-- INSERT INTO metadata (path, access_level)
-- VALUES ('/example/path', 2);

INSERT INTO metadata (path, access_level)
VALUES ('assets/documents/Case details', 1);

INSERT INTO metadata (path, password)
VALUES ("assets/documents/Victim's Computer",
"ef0f02f46be43386f1fa357c03526f92480cc8b4221e9ba092b4d64afdf4c2de");

-- Tags
INSERT INTO tags (name) VALUES ('time_left_office');  -- id 1
INSERT INTO tags (name) VALUES ('cause_of_death');    -- id 2
INSERT INTO tags (name) VALUES ('victim_location');   -- id 3
INSERT INTO tags (name) VALUES ('suspect');           -- id 4

-- Metadata entries for individual documents (needed for FK constraint)
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/Case details/Autopsy');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/Case details/Chen Profile.txt');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/Case details/Profiles.txt');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/Case details/Victim chatlogs/Chat with robert');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/Case details/Badge Access Log');

-- Autopsy
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/Case details/Autopsy', 2, 'Gunshot wound to the head');

-- Chen Profile
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/Case details/Chen Profile.txt', 3, 'Riverside Towers, Apt 12B');

-- Profiles
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/Case details/Profiles.txt', 4, 'Robert Zhang');

-- Chat with Robert: Marcus claims he left at 6:23 PM
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/Case details/Victim chatlogs/Chat with robert', 1, '6:23 PM');

-- Badge Access Log: system records show he badged out at 7:47 PM (contradicts chat)
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/Case details/Badge Access Log', 1, '7:47 PM');

-- Test contradiction
INSERT INTO contradictions (doc_path_a, doc_path_b, tag_id)
VALUES (
    'assets/documents/Case details/Victim chatlogs/Chat with robert',
    'assets/documents/Case details/Badge Access Log',
    1
);