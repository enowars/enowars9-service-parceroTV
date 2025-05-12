use std::{thread::sleep, time::Duration};

use actix_web::{error, web, Error};
use rusqlite::Statement;
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;


