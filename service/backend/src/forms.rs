use actix_multipart::form::{tempfile::TempFile, MultipartForm, text::Text};
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

use serde_qs;


#[derive(Deserialize)]
pub struct FormInput {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CommentForm {
    pub comment: String,
    pub video_id: i32,
}

#[derive(Deserialize)]
pub struct UpdateAboutForm {
    pub about: String,
}

#[derive(Debug, Deserialize)]
pub struct VideoInfo {
    pub name: String,
    pub description: String,
    pub is_private: u32,
}


#[derive(Debug, MultipartForm)]
pub struct VideoForm {
    pub name: Text<String>,
    pub description: Text<String>,
    pub is_private: Text<u32>,
    #[multipart(limit = "2MB")]
    pub file: TempFile,
    #[multipart(limit = "2MB")]
    pub thumbnail: TempFile,
    pub location: Text<String>,
}

#[derive(Debug, MultipartForm)]
pub struct ShortsForm{
    pub name: Text<String>,
    pub description: Text<String>,
    #[multipart(limit = "2MB")]
    pub file: TempFile,
    pub captions: Text<String>,
    pub translate_to_spanish: Text<bool>,
    pub duration: Text<f64>,
}



#[derive(Debug, Deserialize)]
pub struct PlaylistForm {
    pub name: String,
    pub description: String,
    pub video_ids: Vec<i32>,
    pub user_ids: Vec<i32>,
    #[serde(default)]
    pub is_private: bool,
}
