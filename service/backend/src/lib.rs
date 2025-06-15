extern crate ffmpeg_next as ffmpeg;
use std::hash::{DefaultHasher, Hash, Hasher};
use tempfile::NamedTempFile;
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::Path;
use regex::Regex;

#[derive(Debug, Hash)]
pub struct VideoMetadata {
    pub title: String,
    pub creator: String,
    pub genre: String,
}



fn sanitize_title(title: &str, is_video: bool) -> &str {
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if re.is_match(title) {
        title
    } else {
        if is_video {
            return "some-video";
        } else {
            return "some-thumbnail";
        }
    }
}


pub fn get_path(is_private: u32, title: &str, file: &NamedTempFile) -> String {
    
    let mut path = String::from("");
    if is_private == 1 {
        path.push_str("private/");
    }
    else {
        path.push_str("videos/");
    }
    path.push_str(sanitize_title(title, true));
    let md = read_metadata(title, file);
    path.push_str(&calculate_hash(&md).to_string());
    path.push_str(".mp4");
    path
}

pub fn get_thumbnail_path(title: &str, file: &NamedTempFile) -> String {
    let mut path = String::from("thumbnails/");
    path.push_str(sanitize_title(title, false));
    let md = read_metadata(title, file);
    path.push_str(&calculate_hash(&md).to_string());
    path.push_str(".png");
    path
}


pub fn save_video(path: &str, mut file: File) -> io::Result<()> {
    let mut data_path = String::from("../data/");
    data_path.push_str(path);
    let target_path = Path::new(&data_path);

    if target_path.exists() {
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut output = File::create(target_path)?;
    output.write_all(&buffer)?;

    Ok(())
}


pub fn save_thumbnail(thumbnail_path: &str, mut thumbnail_file: File) -> io::Result<()> {
    let mut data_path = String::from("../data/");
    data_path.push_str(thumbnail_path);
    let target_path = Path::new(&data_path);

    if target_path.exists() {
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buffer = Vec::new();
    thumbnail_file.read_to_end(&mut buffer)?;
    let mut output = File::create(target_path)?;
    output.write_all(&buffer)?;

    Ok(())
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}



pub fn read_metadata(title: &str, file: &NamedTempFile) -> VideoMetadata {
    ffmpeg::init().unwrap();

    let path = file.path();
    let mut genre = String::from("unknown");
    let mut creator = String::from("unknown");
    let mut title_override = title.to_string();

    match ffmpeg::format::input(path) {
        Ok(context) => {
            for (k, v) in context.metadata().iter() {
                match k.to_lowercase().as_str() {
                    "title" => title_override = v.to_string(),
                    "artist" => creator = v.to_string(),
                    "genre" => genre = v.to_string(),
                    _ => {},
                }
            }
        }
        Err(_) => {
        }
    }

    VideoMetadata {
        title: title_override,
        creator,
        genre,
    }
}
