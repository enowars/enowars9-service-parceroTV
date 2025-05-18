use std::{thread::sleep, time::Duration};

use actix_web::{error, web, Error};
use rusqlite::{Statement, Result, params};
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub async fn get_db_conn(pool: web::Data<Pool>) -> Result<Connection, Error> {
    let pool = pool.clone();
    web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)
}

/*User SQL-Statements*/
pub fn create_user(conn: Connection, name: &str, password: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO users (name, password) VALUES (?1, ?2)",
        (name,password),
    )?;
    println!("{name}{password}");
    Ok(())
}

pub fn get_all_videos(conn: Connection) -> Result<Vec<VideoInfo>> {
    let mut stmt = conn.prepare("SELECT videoid, name, description, thumbnail_path, path FROM videos WHERE is_private = 0")?;

    let videos = stmt
        .query_map([], |row| {
            Ok(VideoInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                thumbnail_path: row.get(3)?,
                path: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    println!("get_all_videos");
    Ok(videos)
}

pub fn select_video_by_path(conn: Connection, path: &str) -> Result<VideoInfo> {
    let mut stmt = conn.prepare("SELECT VideoId, name, description, thumbnail_path, path FROM videos WHERE path = (?1) ORDER BY videoID LIMIT 1")?;
    let video: VideoInfo = stmt.query_row(
    params![path],
    |row| {
        Ok(VideoInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            thumbnail_path: row.get(3)?,
            path: row.get(4)?,
        })
    })?;
    Ok(video)
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

pub fn select_password(conn:Connection, name: &str) -> Result<String> {
    let password:String = conn.query_row(
        "SELECT password FROM users WHERE name = (?1)",
        params![name],
        |row| row.get(0),
    )?;
    println!("{name}{password}");
    Ok(password)
}


pub fn update_about_user(conn: Connection, about:&str, name:&str) -> Result<()> {
    conn.execute(
        "UPDATE users SET about = (?1) WHERE name = (?2)",
        (about, name),
    )?;
    Ok(())
}



pub fn insert_video(conn: Connection, name:&str, description: &str, path: &str, thumbnail_path:&str, user_id: &u32, is_private: &u32) -> Result<()> {
    println!("Insert {} {} {}", name, description, path);
    conn.execute(
        "INSERT INTO videos (name, description, path, thumbnail_path, UserID, is_private) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (name, description, path, thumbnail_path, user_id, is_private)
    )?;
    
    Ok(())
}

pub fn create_comment(conn:Connection) -> Result<()> {
   
    Ok(())

}


//Structs
#[derive(Debug, serde::Serialize)]
pub struct VideoInfo{
    pub id: i32,
    pub name: String,
    pub description: String,
    pub thumbnail_path: String,
    pub path: String,
}