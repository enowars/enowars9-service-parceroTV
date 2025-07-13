use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use uuid::Uuid;

use crate::spanish_dictionary::SPANISH_WORDS;
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::num::NonZeroU64;

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

pub fn save_caption(
    captions: &str,
    translate_to_spanish: bool,
    duration: f64,
) -> Result<String, std::io::Error> {
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
    println!("duration: {}", duration);

    let mut buffer = generate_vtt(captions, translate_to_spanish, duration);
    let mut output = File::create(saving_path)?;
    output.write_all(&buffer)?;
    Ok(path.to_string())
}

pub fn generate_vtt(captions: &str, translate: bool, duration: f64) -> Vec<u8> {
    let processed_text = if translate {
        translate_to_spanish(captions, duration)
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
            format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
        };

        writeln!(buffer, "{}", i + 1).unwrap();
        writeln!(buffer, "{} --> {}", format_time(start), format_time(end)).unwrap();
        writeln!(buffer, "{}", text).unwrap();
        writeln!(buffer).unwrap(); // Blank line
    }

    buffer
}


fn translate_to_spanish(captions: &str, duration: f64) -> String {
    let transform_stream = get_transform_stream(captions.as_bytes(), duration);

    println!("Original length of bytes in captions: {}", captions.len());
    println!("Length of transform_stream: {}", transform_stream.len());
    let mut bits: u32 = 0;
    let mut bits_count: u8 = 0;
    let mut words = Vec::new();

    for &byte in &transform_stream {
        bits = (bits << 8) | (byte as u32);
        bits_count += 8;
        println!(
            "byte: {} Current bits: {:032b}, bits_count: {}",
            byte, bits, bits_count
        );

        while bits_count >= 12 {
            bits_count -= 12;
            let idx = ((bits >> bits_count) & 0xFFF) as usize;
            println!("Index: {}", idx);
            words.push(SPANISH_WORDS[idx]);
        }
    }

    if bits_count > 0 {
        let idx = ((bits << (12 - bits_count)) & 0xFFF) as usize;
        println!(" Current bits: {:032b}, bits_count: {}", bits, bits_count);
        println!("Index: {}", idx);
        words.push(SPANISH_WORDS[idx]);
    }

    words.join(" ")
}

fn get_transform_stream(words: &[u8], duration: f64) -> Vec<u8> {
    let ms = (duration * 1000.0).round() as u64;
    let ms = NonZeroU64::new(ms).unwrap_or(NonZeroU64::new(1).unwrap());
    let mut seed = [0u8; 32];
    seed[..8].copy_from_slice(&ms.get().to_le_bytes());
    println!("Seed: {:?}", seed);
    let mut rng = ChaCha20Rng::from_seed(seed);

    words.iter()
        .map(|&b| {
            let full = (rng.next_u64());
            let byte = (full & 0xFF) as u8;
            b ^ byte
        })
        .collect()
}
