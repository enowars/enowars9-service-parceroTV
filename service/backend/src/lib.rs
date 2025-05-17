extern crate ffmpeg_next as ffmpeg;
use std::hash::{DefaultHasher, Hash, Hasher};
use tempfile::NamedTempFile;

pub fn get_path(is_private: u32, title: &str, file: NamedTempFile) -> String {
    
    let mut path = String::from("");
    if is_private == 1 {
        path.push_str("private/");
    }
    else {
        path.push_str("videos/");
    }
    path.push_str(title);
    let md = read_metadata(title, file);
    path.push_str(&calculate_hash(&md).to_string());
    path.push_str(".mp4");
    println!("The path is {}", path);
    print_md(&md);
    path
}

use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::Path;

pub fn save_video(path: &str, mut file: File) -> io::Result<()> {
    let mut data_path = String::from("../data/");
    data_path.push_str(path);
    let target_path = Path::new(&data_path);

    if target_path.exists() {
        println!("File already exists at path: {}", path);
        return Ok(());
    }

    // Ensure the parent directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?; // read from the input file

    let mut output = File::create(target_path)?;
    output.write_all(&buffer)?; // write to the output path

    println!("File saved at: {}", path);
    Ok(())
}


fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug, Hash)]
pub struct VideoMetadata {
    pub title: String,
    pub duration: String,
    pub info: String,
    pub creation_time: String,
}

pub fn read_metadata(title: &str, file: NamedTempFile) -> VideoMetadata {
    ffmpeg::init().unwrap();

    let path = file.path().to_str().unwrap();
    let mut info = String::new();
    let mut duration = String::from("unknown");
    let mut creation_time = String::from("unknown");
    let mut title_override = title.to_string();

    match ffmpeg::format::input(path) {
        Ok(context) => {
            for (k, v) in context.metadata().iter() {
                match k.to_lowercase().as_str() {
                    "title" => title_override = v.to_string(),
                    "duration" => duration = v.to_string(),
                    "info" => info = v.to_string(),
                    "creation_time" => creation_time = v.to_string(),
                    _ => {},
                }
            }
        }
        Err(_) => {
        }
    }

    VideoMetadata {
        title: title_override,
        duration,
        info,
        creation_time,
    }
}



//Debug Function
fn print_md(md :&VideoMetadata) {
    println!("title: {} \nduration: {} \ninfo: {} \ncreation_time: {} \n", md.title, md.duration, md.info, md.creation_time)
}