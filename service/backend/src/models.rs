use diesel::prelude::*;
use crate::schema::users;
use crate::schema::comments;
use crate::schema::videos;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub UserID: i32,
    pub name: String,
    pub password: String,
    pub about: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub password: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::videos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Video {
    pub VideoID: i32,
    pub name: String,
    pub description: Option<String>,
    pub source: Vec<u8>,
    pub UserID: Option<i32>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::comments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Comment {
    pub CommentsID: i32,
    pub comment: String,
    pub VideoID: Option<i32>,
    pub UserID: Option<i32>,
}



