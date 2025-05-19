CREATE TABLE users (
    UserID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL CHECK(length(name) <= 20),
    password TEXT NOT NULL CHECK(length(password) <= 200),
    about TEXT CHECK(length(about) <= 2000)
);

CREATE TABLE videos(
    VideoID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) <= 20),
    description TEXT NOT NULL CHECK(length(description) <= 1000),
    path TEXT NOT NULL,
    thumbnail_path TEXT NOT NULL,
    UserID INTEGER,
    is_private INTEGER NOT NULL,
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE
);

CREATE TABLE comments(
    CommentsID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    comment TEXT NOT NULL,
    UserID INTEGER NOT NULL,
    VideoID INTEGER NOT NULL,
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE,
    FOREIGN KEY (VideoID) REFERENCES videos(VideoID) ON DELETE CASCADE
);

CREATE VIEW public_videos AS
SELECT *
FROM videos
WHERE is_private = 0;
