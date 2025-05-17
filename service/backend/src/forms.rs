use actix_multipart::form::{tempfile::TempFile, MultipartForm, text::Text};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormInput {
    pub username: String,
    pub password: String,
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
    pub file: TempFile,
}
