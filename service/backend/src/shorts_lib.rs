use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use uuid::Uuid;

pub fn save_short(mut file_to_save: File) -> Result<String, std::io::Error> {
    let id = Uuid::new_v4();
    let mut path = String::from("/shorts/");
    path.push_str(&id.to_string());
    path.push_str(".mp4");

    let mut saving_path = String::from("../data");
    saving_path.push_str(&path);
    let saving_path = Path::new(&saving_path);

    if let Some(parent) = saving_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buffer = Vec::new();
    file_to_save.read_to_end(&mut buffer)?;
    let mut output = File::create(saving_path)?;
    output.write_all(&buffer)?;
    Ok(path.to_string())
}

pub fn save_caption(captions: &str, translate_to_spanish: bool, duration: f64) -> Result<String, std::io::Error> {
    let id = Uuid::new_v4();
    let mut path = String::from("/vtt/");
    path.push_str(&id.to_string());
    path.push_str(".vtt");

    let mut saving_path = String::from("../data");
    saving_path.push_str(&path);
    let saving_path = Path::new(&saving_path);

    if let Some(parent) = saving_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut buffer = generate_vtt(captions, translate_to_spanish, duration);
    let mut output = File::create(saving_path)?;
    output.write_all(&buffer)?;
    Ok(path.to_string())
}

pub fn generate_vtt(captions: &str, translate_to_spanish: bool, duration: f64) -> Vec<u8> {
    let processed_text = if translate_to_spanish {
        translate_caption_to_spanish(captions)
    } else {
        captions.to_string()
    };

    let num_parts = duration.round().clamp(5.0, 10.0).round() as usize;

    let words: Vec<&str> = processed_text.split_whitespace().collect();
    let mut parts = Vec::new();
    let chunk_size = (words.len() as f64 / num_parts as f64).ceil() as usize;

    for chunk in words.chunks(chunk_size) {
        parts.push(chunk.join(" "));
    }

    let mut buffer = Vec::new();
    writeln!(buffer, "WEBVTT").unwrap();
    writeln!(buffer).unwrap();

    for (i, text) in parts.iter().enumerate() {
        let start = duration * (i as f64) / num_parts as f64;
        let end = duration * ((i + 1) as f64) / num_parts as f64;

        let format_time = |sec: f64| {
            let total_ms = (sec * 1000.0).round() as u64;
            let h = total_ms / 3600000;
            let m = (total_ms % 3600000) / 60000;
            let s = (total_ms % 60000) / 1000;
            let ms = total_ms % 1000;
            format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
        };

        writeln!(buffer, "{}", i + 1).unwrap();
        writeln!(buffer, "{} --> {}", format_time(start), format_time(end)).unwrap();
        writeln!(buffer, "{}", text).unwrap();
        writeln!(buffer).unwrap(); // Blank line
        println!("{}",text);
    }

    buffer
}

fn translate_caption_to_spanish(captions: &str) -> String{String::from("filler")}

//fn translate_word_to_spanish(word: &str);
