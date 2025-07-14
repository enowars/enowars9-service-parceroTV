CREATE TABLE IF NOT EXISTS users (
    UserID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL CHECK(length(name) <= 20),
    password TEXT NOT NULL CHECK(length(password) <= 200),
    created_at TEXT DEFAULT (datetime('now')),
    about TEXT CHECK(length(about) <= 2000)
);

CREATE TABLE IF NOT EXISTS videos(
    VideoID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) <= 200),
    description TEXT NOT NULL CHECK(length(description) <= 2000),
    path TEXT NOT NULL,
    thumbnail_path TEXT NOT NULL,
    UserID INTEGER NOT NULL,
    is_private INTEGER NOT NULL,
    location TEXT,
    likes INTEGER NOT NULL DEFAULT(0),
    dislikes INTEGER NOT NULL DEFAULT(0),
    clicks INTEGER NOT NULL DEFAULT(0),
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS has_liked(
    UserID INTEGER NOT NULL,
    VideoID INTEGER NOT NULL,
    like_status INTEGER NOT NULL CHECK(like_status IN (0, 1)), -- 0 for dislike, 1 for like
    PRIMARY KEY (UserID, VideoID),
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE,
    FOREIGN KEY (VideoID) REFERENCES videos(VideoID) ON DELETE CASCADE
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

CREATE TABLE IF NOT EXISTS playlist(
    PlaylistID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(name) <= 200),
    description TEXT NOT NULL CHECK(length(description) <= 2000),
    is_private INTEGER NOT NULL,
    owner_userID INTEGER NOT NULL,
    FOREIGN KEY (owner_userID) REFERENCES users(UserID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS access_rights_playlist(
    PlaylistID INTEGER NOT NULL,
    userID INTEGER NOT NULL,
    PRIMARY KEY (PlaylistID, userID),
    FOREIGN KEY (playlistID) REFERENCES playlist(playlistID) ON DELETE CASCADE,
    FOREIGN KEY (userID) REFERENCES users(userID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS videos_in_playlist(
    PlaylistID INTEGER NOT NULL,
    videoID INTEGER NOT NULL,
    PRIMARY KEY (PlaylistID, videoID),
    FOREIGN KEY (playlistID) REFERENCES playlist(playlistID) ON DELETE CASCADE,
    FOREIGN KEY (videoID) REFERENCES videos(videoID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS shorts(
    ShortID INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL CHECK(length(name) <= 200),
    description TEXT NOT NULL CHECK(length(description) <= 2000),
    path TEXT NOT NULL,
    caption_path TEXT,
    original_captions TEXT,
    UserID INTEGER NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (UserID) REFERENCES users(UserID) ON DELETE CASCADE
);

CREATE VIEW IF NOT EXISTS public_videos AS
SELECT *
FROM videos
WHERE is_private = 0;

CREATE VIEW IF NOT EXISTS private_videos AS
SELECT *
FROM videos
WHERE is_private = 1;

-- Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_videos_is_private ON videos(is_private);
CREATE INDEX IF NOT EXISTS idx_videos_userid_created_at ON videos(userID, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_videos_private_userid ON videos(is_private, userID);
CREATE UNIQUE INDEX IF NOT EXISTS ux_videos_path ON videos(path);

CREATE INDEX IF NOT EXISTS idx_comments_video_created ON comments(VideoID, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comments_userid ON comments(UserID);

CREATE INDEX IF NOT EXISTS idx_shorts_created_at ON shorts(created_at DESC);

CREATE UNIQUE INDEX IF NOT EXISTS ux_users_name ON users(name);

PRAGMA foreign_keys = ON;