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
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/Case details/Mediflow Systems/Badge Access Log', 1, '7:47 PM');



-- Tags (appending to existing ids 1-4)
INSERT INTO tags (name) VALUES ('probation_period');  -- id 5
INSERT INTO tags (name) VALUES ('non_compete');       -- id 6
INSERT INTO tags (name) VALUES ('start_date');        -- id 7
INSERT INTO tags (name) VALUES ('salary');            -- id 8
INSERT INTO tags (name) VALUES ('vacation_days');     -- id 9
INSERT INTO tags (name) VALUES ('working_hours');     -- id 10
INSERT INTO tags (name) VALUES ('governing_law');     -- id 11
INSERT INTO tags (name) VALUES ('job_title');         -- id 12
INSERT INTO tags (name) VALUES ('signing_date');      -- id 13

-- Metadata entries
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/On boarding/Offer Letter');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/On boarding/Employment Contract');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/On boarding/NDA');
INSERT OR IGNORE INTO metadata (path) VALUES ('assets/documents/On boarding/Employee Handbook');

-- job_title
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 12, 'Forensic Intelligence Analyst');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 12, 'Forensic Intelligence Analyst');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/NDA', 12, 'Forensic Intelligence Analyst');

-- signing_date
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 13, 'March 3 2025');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 13, 'March 3 2025');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/NDA', 13, 'March 3 2025');

-- salary
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 8, '$5,400/month');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 8, '$5,400/month');

-- vacation_days
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 9, '15 days');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 9, '15 days');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employee Handbook', 9, '15 days');

-- working_hours
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 10, '40 hours/week');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employee Handbook', 10, '40 hours/week');

-- governing_law
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 11, 'District of Columbia');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/NDA', 11, 'District of Columbia');

-- start_date
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 7, 'Monday April 14 2025');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 7, 'Monday April 14 2025');

-- probation_period
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 5, '30 days');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 5, '90 days');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employee Handbook', 5, '90 days');

-- non_compete
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Offer Letter', 6, 'none');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/Employment Contract', 6, '12 months');
INSERT INTO metadata_tags (path, tag_id, value) VALUES ('assets/documents/On boarding/NDA', 6, '6 months');

-- e699e0e29661c368c8fb9b1aa50e350926af0d850dc1967ecefb41b5d12a5fcd
INSERT INTO metadata (path, password)
VALUES ("assets/documents/On boarding",
"e699e0e29661c368c8fb9b1aa50e350926af0d850dc1967ecefb41b5d12a5fcd");
