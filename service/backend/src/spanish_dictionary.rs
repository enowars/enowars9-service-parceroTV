use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;

static RAW_WORDS: &str = include_str!("spanish_words.txt");

const PER_LETTER: usize = 4096 / 26;

pub static SPANISH_WORDS: Lazy<[&'static str; 4096]> = Lazy::new(|| {
    let mut groups: HashMap<char, Vec<&str>> = HashMap::new();
    for word in RAW_WORDS.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(ch) = word.chars().next() {
            let lc = ch.to_ascii_lowercase();
            if ('a'..='z').contains(&lc) {
                groups.entry(lc).or_default().push(word);
            }
        }
    }

    let mut used = HashSet::new();
    let mut arr = Vec::with_capacity(4096);

    for letter in 'a'..='z' {
        if let Some(bucket) = groups.get(&letter) {
            for &w in bucket.iter().take(PER_LETTER) {
                if used.insert(w) {
                    arr.push(w);
                }
            }
        }
    }
    for word in RAW_WORDS.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if arr.len() == 4096 { break; }
        if used.insert(word) {
            arr.push(word);
        }
    }

    assert!(arr.len() == 4096, "To translate to future spanish, the future words file must contain exactly 4096 unique words.");
    arr.try_into().expect("Vec length must be 4096")
});