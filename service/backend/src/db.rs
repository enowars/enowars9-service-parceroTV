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



pub fn create_video(conn: Connection, name:&str, description: &str, source: &Vec<u8>, user_id: &u32) -> Result<()> {
    conn.execute(
        "INSERT INTO videos (name, description, source, UserID) VALUES (?1, ?2, ?3, ?4)",
        (name, description, source, user_id)
    )?;
    Ok(())
}

pub fn create_comment(conn:Connection) -> Result<()> {
   
    Ok(())

}