CREATE TABLE IF NOT EXISTS users (
    UserID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL CHECK(length(name) <= 20),
    password TEXT NOT NULL CHECK(length(password) <= 200),
    about TEXT CHECK(length(about) <= 2000)
);

CREATE TABLE IF NOT EXISTS videos(
    VideoID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) <= 200),
    description TEXT NOT NULL CHECK(length(description) <= 2000),
    path TEXT NOT NULL,
    thumbnail_path TEXT NOT NULL,
    UserID INTEGER,
    is_private INTEGER NOT NULL,
    location TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS comments(
    CommentsID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    comment TEXT NOT NULL,
    UserID INTEGER NOT NULL,
    VideoID INTEGER NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE,
    FOREIGN KEY (VideoID) REFERENCES videos(VideoID) ON DELETE CASCADE
);

CREATE VIEW IF NOT EXISTS public_videos AS
SELECT *
FROM videos
WHERE is_private = 0;

CREATE VIEW IF NOT EXISTS private_videos AS
SELECT *
FROM videos
WHERE is_private = 1;
