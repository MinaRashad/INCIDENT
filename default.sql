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
    access_level INTEGER NOT NULL
);


-- INSERT INTO metadata (path, access_level)
-- VALUES ('/example/path', 2);

INSERT INTO metadata (path, access_level)
VALUES ('assets/documents/Case details', 1);

INSERT INTO metadata (path, password)
VALUES ("assets/documents/Victim's Computer", 
"9bb840df7f699d6547f49fd6f9ed17b2d9dd34148f0b01e798d7c51da897ea1a");