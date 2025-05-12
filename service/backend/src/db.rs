use std::{thread::sleep, time::Duration};

use actix_web::{error, web, Error};
use rusqlite::{Statement, Result};
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;


pub fn create_user(conn: Connection, name: &str, password: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO users (name, password) VALUES (?1, ?2)",
        (name,password),
    )?;
    println!("{name}{password}");
    Ok(())
}