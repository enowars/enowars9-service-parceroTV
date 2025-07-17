use std::{thread::sleep, time::Duration};

use actix_web::{Error, error, web};
use ffmpeg_next::chroma::location;

use rusqlite::{OptionalExtension, Result, Statement, params, params_from_iter};
use serde::{Deserialize, Serialize};
use rusqlite::types::ToSql;

use crate::playlist;
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub async fn get_db_conn(pool: &web::Data<Pool>) -> Result<Connection, Error> {
    let pool = pool.clone();
    web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)
}

/*User SQL-Statements*/
pub fn create_user(conn: Connection, name: &str, password: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO users (name, password) VALUES (?1, ?2)",
        (name, password),
    )?;
    Ok(())
}

pub fn get_all_videos(conn: Connection) -> Result<Vec<VideoInfo>> {
    let mut stmt = conn.prepare("SELECT videoid, name, description, thumbnail_path, path, is_private, location, userId, likes, dislikes, clicks FROM videos WHERE is_private = 0")?;

    let videos = stmt
        .query_map([], |row| {
            Ok(VideoInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                thumbnail_path: row.get(3)?,
                path: row.get(4)?,
                is_private: row.get(5)?,
                location: row.get(6)?,
                userId: row.get(7)?,
                likes: row.get(8)?,
                dislikes: row.get(9)?,
                clicks: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(videos)
}

pub fn select_my_videos(conn: Connection, user_id: &i32) -> Result<Vec<VideoInfo>> {
    let mut stmt = conn.prepare("SELECT videoid, name, description, thumbnail_path, path, is_private, location, userId, likes, dislikes, clicks FROM videos WHERE userid = ?1 ORDER BY created_at DESC")?;

    let videos = stmt
        .query_map([user_id], |row| {
            Ok(VideoInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                thumbnail_path: row.get(3)?,
                path: row.get(4)?,
                is_private: row.get(5)?,
                location: row.get(6)?,
                userId: row.get(7)?,
                likes: row.get(8)?,
                dislikes: row.get(9)?,
                clicks: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(videos)
}

pub fn select_videos_by_userid(conn: Connection, user_id: i32) -> Result<Vec<VideoInfo>> {
    let mut stmt = conn.prepare("SELECT videoid, name, description, thumbnail_path, path, is_private, location, userId, likes, dislikes, clicks FROM videos WHERE is_private = 0 AND userid = ?1")?;

    let videos = stmt
        .query_map([user_id], |row| {
            Ok(VideoInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                thumbnail_path: row.get(3)?,
                path: row.get(4)?,
                is_private: row.get(5)?,
                location: row.get(6)?,
                userId: row.get(7)?,
                likes: row.get(8)?,
                dislikes: row.get(9)?,
                clicks: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(videos)
}

pub fn select_private_videos_by_userid(
    conn: Connection,
    user_id: i32,
) -> Result<Vec<VideoInfoPrivate>> {
    let mut stmt = conn.prepare("SELECT videoid, name, thumbnail_path, location, userId FROM videos WHERE is_private = 1 AND userid = ?1")?;

    let videos = stmt
        .query_map([user_id], |row| {
            Ok(VideoInfoPrivate {
                id: row.get(0)?,
                name: row.get(1)?,
                thumbnail_path: row.get(2)?,
                location: row.get(3)?,
                userId: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(videos)
}

pub fn select_video_by_path(conn: Connection, path: &str) -> Result<VideoInfo> {
    let mut stmt = conn.prepare("SELECT VideoId, name, description, thumbnail_path, path, is_private, location, userId, likes, dislikes, clicks FROM videos WHERE path = (?1) ORDER BY videoID LIMIT 1")?;
    let video: VideoInfo = stmt.query_row(params![&path], |row| {
        Ok(VideoInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            thumbnail_path: row.get(3)?,
            path: row.get(4)?,
            is_private: row.get(5)?,
            location: row.get(6)?,
            userId: row.get(7)?,
            likes: row.get(8)?,
            dislikes: row.get(9)?,
            clicks: row.get(10)?,
        })
    })?;
    Ok(video)
}

pub fn select_shorts(conn: Connection, user_id: &i32) -> Result<Vec<ShortInfo>> {
    let mut stmt = conn.prepare(
        "SELECT ShortID, name, description, path, caption_path, CASE WHEN userID = ?1 THEN original_captions ELSE NULL END, UserID FROM shorts ORDER BY created_at DESC",
    )?;

    let shorts = stmt
        .query_map([user_id], |row| {
            Ok(ShortInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                path: row.get(3)?,
                caption_path: row.get(4)?,
                original_captions: row.get(5)?,
                user_id: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(shorts)
}

pub fn select_comments_by_video_id(
    conn: Connection,
    video_id: &i32,
) -> Result<Vec<CommentWithUserInfo>> {
    let mut stmt = conn.prepare(
        "SELECT 
    comments.CommentsID,
    comments.comment,
    comments.UserID,
    users.name AS username,
    comments.created_at
FROM comments
JOIN users ON comments.UserID = users.UserID
WHERE comments.VideoID = ?1
ORDER BY comments.created_at DESC;",
    )?;

    let comments = stmt
        .query_map([video_id], |row| {
            Ok(CommentWithUserInfo {
                comment_id: row.get(0)?,
                comment: row.get(1)?,
                user_id: row.get(2)?,
                username: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(comments)
}

pub fn user_has_permission(conn: &Connection, user_id: &i32, path: &str) -> Result<bool> {
    let result: Result<i32> = conn.query_row(
        "SELECT userID FROM videos WHERE userID = ?1 AND path = ?2;",
        params![user_id, path],
        |row| row.get(0),
    );

    match result {
        Ok(_) => Ok(true),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn select_user_id(conn: Connection, name: &str) -> Result<i32> {
    let user_id: i32 = conn.query_row(
        "SELECT UserID FROM users WHERE name = (?1)",
        params![name],
        |row| row.get(0),
    )?;

    Ok(user_id)
}

pub fn select_user_info(conn: Connection, id: &i32) -> Result<UserInfo> {
    let user_info = conn.query_row(
        "SELECT UserID, name, about FROM users WHERE userID = (?1)",
        params![id],
        |row| {
            Ok(UserInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                about: row.get(2)?,
            })
        },
    )?;

    Ok(user_info)
}

pub fn select_user_info_with_name(conn: Connection, id: &String) -> Result<UserInfo> {
    let user_info = conn.query_row(
        "SELECT UserID, name, about FROM users WHERE name = (?1)",
        params![id],
        |row| {
            Ok(UserInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                about: row.get(2)?,
            })
        },
    )?;

    Ok(user_info)
}

pub fn select_password(conn: Connection, name: &str) -> Result<Option<String>> {
    let password = conn
        .query_row(
            "SELECT password FROM users WHERE name = (?1)",
            params![name],
            |row| row.get(0),
        )
        .optional()?;
    Ok(password)
}

pub fn update_about_user(conn: Connection, about: &str, user_id: &i32) -> Result<()> {
    conn.execute(
        "UPDATE users SET about = (?1) WHERE userID = (?2)",
        (about, user_id),
    )?;
    Ok(())
}

pub fn insert_video(
    conn: Connection,
    name: &str,
    description: &str,
    path: &str,
    thumbnail_path: &str,
    user_id: &u32,
    is_private: &u32,
    location: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO videos (name, description, path, thumbnail_path, UserID, is_private, location) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (name, description, path, thumbnail_path, user_id, is_private, location)
    )?;

    Ok(())
}

pub fn insert_short(
    conn: Connection,
    name: &str,
    description: &str,
    path: &str,
    caption_path: Option<&str>,
    original_captions: Option<&str>,
    user_id: &u32,
) -> Result<()> {
    conn.execute(
        "INSERT INTO shorts (name, description, path, caption_path, original_captions, UserID) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (name, description, path, caption_path, original_captions, user_id)
    )?;

    Ok(())
}

pub fn create_comment(conn: Connection, comment: &str, user_id: &i32, videoid: &i32) -> Result<()> {
    conn.execute(
        "INSERT INTO comments (comment, userid, videoid) VALUES (?1, ?2, ?3)",
        (comment, user_id, videoid),
    )?;
    Ok(())
}

pub fn is_video_private(conn: &Connection, video_id: &i32) -> Result<bool> {
    let is_private: i32 = conn.query_row(
        "SELECT CASE WHEN EXISTS (SELECT videoID FROM videos WHERE videoID = ?1 AND is_private = 1) THEN 1 ELSE 0 END;",
         params![video_id], |row| row.get(0),)?;

    Ok(is_private == 1)
}

//Video Statistics
pub fn get_like_status_db(conn: &Connection, user_id: &i32, video_id: &i32) -> Result<LikeStatus> {
    let result: Result<i32, rusqlite::Error> = conn.query_row(
        "SELECT like_status FROM has_liked WHERE UserID = ?1 AND VideoID = ?2",
        params![user_id, video_id],
        |row| row.get(0),
    );

    match result {
        Ok(1) => Ok(LikeStatus {
            status: "liked".to_string(),
        }),
        Ok(0) => Ok(LikeStatus {
            status: "disliked".to_string(),
        }),
        Ok(_) => Ok(LikeStatus {
            status: "none".to_string(),
        }),
        Err(_) => Ok(LikeStatus {
            status: "none".to_string(),
        }),
    }
}

pub fn update_like_db(conn: &Connection, user_id: &i32, video_id: &i32) -> Result<bool> {
    let result: Result<i32, rusqlite::Error> = conn.query_row(
        "SELECT like_status FROM has_liked WHERE UserID = ?1 AND VideoID = ?2",
        params![user_id, video_id],
        |row| row.get(0),
    );

    match result {
        Ok(1) => Ok(false),
        Ok(0) => {
            conn.execute(
                "UPDATE has_liked SET like_status = 1 WHERE UserID = ?1 AND VideoID = ?2",
                params![user_id, video_id],
            )?;
            conn.execute(
                "UPDATE videos SET likes = likes + 1 WHERE videoID = ?1",
                params![video_id],
            )?;
            conn.execute(
                "UPDATE videos SET dislikes = dislikes - 1 WHERE videoID = ?1",
                params![video_id],
            )?;
            Ok(true)
        }
        Ok(_) => Ok(false),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            conn.execute(
                "INSERT INTO has_liked (UserID, VideoID, like_status) VALUES (?1, ?2, 1)",
                params![user_id, video_id],
            )?;
            conn.execute(
                "UPDATE videos SET likes = likes + 1 WHERE videoID = ?1",
                params![video_id],
            )?;
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

pub fn update_dislike_db(conn: &Connection, user_id: &i32, video_id: &i32) -> Result<bool> {
    let result: Result<i32, rusqlite::Error> = conn.query_row(
        "SELECT like_status FROM has_liked WHERE UserID = ?1 AND VideoID = ?2",
        params![user_id, video_id],
        |row| row.get(0),
    );

    match result {
        Ok(0) => Ok(false), // Already disliked
        Ok(1) => {
            conn.execute(
                "UPDATE has_liked SET like_status = 0 WHERE UserID = ?1 AND VideoID = ?2",
                params![user_id, video_id],
            )?;
            conn.execute(
                "UPDATE videos SET likes = likes - 1 WHERE videoID = ?1",
                params![video_id],
            )?;
            conn.execute(
                "UPDATE videos SET dislikes = dislikes + 1 WHERE videoID = ?1",
                params![video_id],
            )?;
            Ok(true)
        }
        Ok(_) => Ok(false),

        Err(rusqlite::Error::QueryReturnedNoRows) => {
            conn.execute(
                "INSERT INTO has_liked (UserID, VideoID, like_status) VALUES (?1, ?2, 0)",
                params![user_id, video_id],
            )?;
            conn.execute(
                "UPDATE videos SET dislikes = dislikes + 1 WHERE videoID = ?1",
                params![video_id],
            )?;
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

pub fn increase_view_count_db(conn: &Connection, video_id: &i32) -> Result<()> {
    conn.execute(
        "UPDATE videos SET clicks = clicks + 1 WHERE videoID = ?1",
        params![video_id],
    )?;
    Ok(())
}

pub fn get_all_users_db(conn: &Connection) -> Result<Vec<UserInfo>> {
    let mut stmt = conn.prepare("SELECT UserID, name, about FROM users LIMIT 50")?;
    let users = stmt
        .query_map([], |row| {
            Ok(UserInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                about: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(users)
}

pub fn get_playlists_public_db(conn: &Connection) -> Result<Vec<PlaylistInfo>> {
    let mut stmt = conn.prepare(
        "SELECT PlaylistID, name, description FROM playlist WHERE is_private = 0",
    )?;

    let playlists = stmt
        .query_map([], |row| {
            Ok(PlaylistInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(playlists)
}

pub fn get_playlists_private_db(conn: &Connection, user_id: &u32) -> Result<Vec<PlaylistInfo>> {
    let mut stmt = conn.prepare(
        "SELECT PlaylistID, name, description FROM playlist WHERE is_private = 1 AND owner_UserID = ?1",
    )?;

    let playlists = stmt
        .query_map([user_id], |row| {
            Ok(PlaylistInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(playlists)
}


pub fn check_videos_private(conn: &Connection, video_ids: &[i32]) -> Result<bool> {
    if video_ids.is_empty() {
        return Ok(false);
    }

    let placeholders = std::iter::repeat("?")
        .take(video_ids.len())
        .collect::<Vec<_>>()
        .join(",");

    let sql = format!(
        "SELECT COUNT(*) FROM videos \
         WHERE is_private = 1 \
           AND videoID IN ({})",
        placeholders,
    );

    let mut stmt = conn.prepare(&sql)?;

    let count: i32 = stmt.query_row(
        params_from_iter(video_ids.iter()),
        |row| row.get(0),
    )?;

    println!("Count of private videos: {}", count);
    Ok(count > 0)
}


pub fn create_playlist_db(
    conn: &Connection,
    name: &str,
    description: &str,
    video_ids: &[i32],
    user_ids: &[i32],
    user_id: &u32,
    is_private: bool,
) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO playlist (name, description, is_private, owner_userID) VALUES (?1, ?2, ?3, ?4)",
    )?;
    stmt.execute(params![name, description, is_private as i32, user_id])?;

    let playlist_id = conn.last_insert_rowid() as i32;

    let placeholders_for_videos_in_playlist = std::iter::repeat("(?, ?)")
    .take(video_ids.len()).collect::<Vec<_>>().join(",");

    let sql_videos_in_playlist = format!("INSERT INTO videos_in_playlist (PlaylistID, videoID) VALUES {}", placeholders_for_videos_in_playlist);
     let mut params: Vec<&dyn ToSql> = Vec::with_capacity(video_ids.len() * 2);
    for vid in video_ids {
        params.push(&playlist_id);
        params.push(vid);
    }

    conn.execute(&sql_videos_in_playlist, params.as_slice())?;
    println!("Successfully Insert vid in playlist");

     let placeholders_for_videos_in_playlist = std::iter::repeat("(?, ?)")
    .take(user_ids.len()).collect::<Vec<_>>().join(",");
    let sql_user_rights_in_playlist = format!("INSERT INTO access_rights_playlist (PlaylistID, UserID) VALUES {}", placeholders_for_videos_in_playlist);
    let mut params: Vec<&dyn ToSql> = Vec::with_capacity(user_ids.len() * 2);
    for uid in user_ids {
        params.push(&playlist_id);
        params.push(uid);
    }

    conn.execute(&sql_user_rights_in_playlist, params.as_slice())?;
    println!("Successfully Inserted UserID");
    Ok(())
}


pub fn get_playlist_db(conn: &Connection, playlist_id: &i32) -> Result<PlaylistInfo> {
    let playlist_info = conn.query_row("SELECT PlaylistID, name, description FROM playlist WHERE PlaylistID = ?1", [playlist_id], |row| {
        Ok(PlaylistInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
        })
    })?;
    Ok(playlist_info)
}

pub fn get_videos_in_playlist_db(conn: &Connection, playlist_id: &i32) -> Result<Vec<VideoInfo>> {
    let mut stmt = conn.prepare("SELECT videoid, name, description, thumbnail_path, path, is_private, location, userId, likes, dislikes, clicks FROM videos WHERE videoID IN (SELECT videoID FROM videos_in_playlist WHERE PlaylistID = ?1)")?;
    let videos = stmt
        .query_map([playlist_id], |row| {
            Ok(VideoInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                thumbnail_path: row.get(3)?,
                path: row.get(4)?,
                is_private: row.get(5)?,
                location: row.get(6)?,
                userId: row.get(7)?,
                likes: row.get(8)?,
                dislikes: row.get(9)?,
                clicks: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(videos)
}

//Structs
#[derive(Debug, serde::Serialize)]
pub struct VideoInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub thumbnail_path: String,
    pub path: String,
    pub is_private: i32,
    pub location: String,
    pub userId: i32,
    pub likes: i32,
    pub dislikes: i32,
    pub clicks: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct VideoInfoPrivate {
    pub id: i32,
    pub name: String,
    pub thumbnail_path: String,
    pub location: String,
    pub userId: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct ShortInfo {
    id: i32,
    name: String,
    description: String,
    path: String,
    caption_path: String,
    original_captions: Option<String>,
    user_id: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub name: String,
    pub about: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CommentWithUserInfo {
    comment_id: i32,
    comment: String,
    user_id: i32,
    username: String,
    created_at: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LikeStatus {
    status: String,
}

#[derive(Debug, serde::Serialize)]
pub struct PlaylistInfo {
    pub id: i32,
    pub name: String,
    pub description: String,
}
