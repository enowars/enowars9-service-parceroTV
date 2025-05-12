pub mod models;
pub mod schema;
use diesel::SelectableHelper;
use diesel::SqliteConnection;
use diesel::RunQueryDsl;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::PooledConnection;

use self::models::{NewUser, User};

pub fn create_user(conn: &mut SqliteConnection, name: &str, password: &str) -> User {
    use crate::schema::users::dsl::*

    let new_user = NewUser { name, password };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)
        .expect("Error saving new user");
    
    let user = users
        .filter(name.eq(name))
        .first::<User>(conn)
        .expect("Error loading person that was just inserted");
    
    Ok(user)
}