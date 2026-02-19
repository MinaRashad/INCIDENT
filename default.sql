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
    PRIMARY KEY (path, tag_id),
    FOREIGN KEY (path) REFERENCES metadata(path) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS player (
    id INTEGER PRIMARY KEY CHECK (id = 0), -- Ensure only one row exists
    name TEXT NOT NULL,
    access_level INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sender      TEXT NOT NULL,      -- NPC name or "player"
    content     TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE npc_dialogue_state (
    npc_name TEXT PRIMARY KEY,
    node TEXT NOT NULL
);

CREATE TABLE conditions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);





-- INSERT INTO metadata (path, access_level)
-- VALUES ('/example/path', 2);

INSERT INTO metadata (path, access_level)
VALUES ('assets/documents/Case details', 1);

INSERT INTO metadata (path, password)
VALUES ("assets/documents/Victim's Computer", 
"ef0f02f46be43386f1fa357c03526f92480cc8b4221e9ba092b4d64afdf4c2de");