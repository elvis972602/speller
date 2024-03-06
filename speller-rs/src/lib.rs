mod string;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use crate::string::PUNCTUATION;

pub struct Speller {
    distance: i32,
    case_sensitive: bool,
    word_frequency: WordFrequency,
}

impl Speller {
    pub fn builder() -> SpellerBuilder {
        SpellerBuilder::new()
    }

    pub fn correct(&self, word: &str) -> Option<String> {
        let word = if self.case_sensitive { word.to_string() } else { word.to_lowercase() };

        let candidates = self.candidates(&word);

        if candidates.is_none() || candidates.as_ref().unwrap().is_empty() {
            return None;
        }

        candidates.unwrap().into_iter().max_by_key(|candidate| {
            // println!("candidate: {:?}, score: {:?}", candidate, self.word_frequency.dictionary.get(candidate).unwrap_or(&0));
            self.word_frequency.dictionary.get(candidate).unwrap_or(&0)
        })
    }

    pub fn candidates(&self, word: &str) -> Option<Vec<String>> {
        if self.known(word) {
            return Some(vec![word.to_string()]);
        }

        let res: Vec<String> = self.edit_distance_1(word);

        let tmp: Vec<String> = self.known_set(res.clone());
        if tmp.len() > 0 {
            return Some(tmp);
        }

        if self.distance == 2 {
            let tmp: Vec<String> = self.known_set(self.edit_distance_alt(res));
            if !tmp.is_empty() {
                return Some(tmp);
            }
        }

        None
    }

    pub fn edit_distance_1(&self, word: &str) -> Vec<String> {
        if !self.should_check(word) {
            return vec![];
        }
        let mut results: Vec<String> = Vec::new();
        let word_char = word.chars().collect::<Vec<char>>();
        let len = word_char.len();

        for i in 0..len {
            let (left, right) = word_char.split_at(i);

            if right.len() > 0 {
                results.push(left.iter().chain(&right[1..]).collect());
            }

            if right.len() > 1 {
                results.push(left.iter().chain(&right[1..2]).chain(&right[0..1]).chain(&right[2..]).collect());
            }

            self.word_frequency.letters.iter().for_each(|c| {
                if right.len() > 1 {
                    results.push(left.iter().chain(&[*c]).chain(&right[1..]).collect());
                }
                results.push(left.iter().chain(&[*c]).chain(right.iter()).collect());
            });
        }

        results
    }

    pub fn edit_distance_alt(&self, words: Vec<String>) -> Vec<String> {
        let tmp_words: Vec<String> = words.into_iter().map(|word| if self.case_sensitive { word } else { word.to_lowercase() }).collect();
        let mut results: Vec<String> = Vec::new();
        for word in tmp_words {
            for w in self.known_set(self.edit_distance_1(&word)) {
                results.push(w);
            }
        }
        results
    }

    fn known(&self, word: &str) -> bool {
        let word = if self.case_sensitive { word.to_string() } else { word.to_lowercase() };
        self.word_frequency.dictionary.contains_key(&word)
    }

    fn known_set(&self, words: Vec<String>) -> Vec<String> {
        words.into_iter().filter(|word| self.known(word) && self.should_check(word)).collect()
    }

    fn should_check(&self, word: &str) -> bool {
        let len = word.len();
        if len == 1 && PUNCTUATION.contains(&word.chars().next().unwrap()) {
            return false;
        }
        if len > self.word_frequency.longest_word + 3 {
            return false;
        }
        if word.to_lowercase() == "nan" {
            return true;
        }
        // if can parse to number, don't check
        if word.parse::<f64>().is_ok() {
            return false;
        }
        true
    }
}

pub struct SpellerBuilder {
    language: Vec<String>,
    local_dictionary: Option<String>,
    distance: i32,
    case_sensitive: bool,
}

impl SpellerBuilder {
    pub fn new() -> SpellerBuilder {
        SpellerBuilder {
            language: vec!["en".to_string()],
            local_dictionary: None,
            distance: 2,
            case_sensitive: false,
        }
    }

    pub fn language(&mut self, language: Vec<String>) -> &mut Self {
        self.language = language;
        self
    }

    pub fn local_dictionary(&mut self, local_dictionary: String) -> &mut Self {
        self.local_dictionary = Some(local_dictionary);
        self
    }

    pub fn distance(&mut self, distance: i32) -> &mut Self {
        self.distance = distance;
        self
    }

    pub fn case_sensitive(&mut self, case_sensitive: bool) -> &mut Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn build(&self) -> Result<Speller, Box<dyn Error>> {
        let mut speller = Speller {
            distance: self.distance,
            case_sensitive: self.case_sensitive,
            word_frequency: WordFrequency::new(self.case_sensitive),
        };

        if let Some(local_dictionary) = &self.local_dictionary {
            let path = Path::new(local_dictionary);
            if path.extension() == Some(OsStr::new("json")) {
                if let Err(e) = speller.word_frequency.load_json(path) {
                    eprintln!("Error loading JSON file: {}", e);
                    return Err(e);
                }
            } else {
                panic!("Local dictionary must be a JSON file")
            }
        }
        if self.language.len() > 0 {
            for lang in &self.language {
                let path = format!("data/{}.json", lang);
                if let Err(e) = speller.word_frequency.load_json(path) {
                    eprintln!("Error loading JSON file: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(speller)
    }
}

pub struct WordFrequency {
    dictionary: HashMap<String, i32>,
    total_words: i32,
    unique_words: i32,
    case_sensitive: bool,
    longest_word: usize,
    letters: HashSet<char>,
}

impl WordFrequency {
    pub fn new(case_sensitive: bool) -> WordFrequency {
        WordFrequency {
            dictionary: HashMap::new(),
            total_words: 0,
            unique_words: 0,
            case_sensitive,
            longest_word: 0,
            letters: HashSet::new(),
        }
    }

    pub fn load_json<T: AsRef<Path>>(&mut self, path: T) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: HashMap<String, i32> = serde_json::from_reader(reader)?;
        data.iter().for_each(|(word, count)| {
            self.add_word(word.to_string(), *count);
        });
        self.update();
        Ok(())
    }

    pub fn add_word(&mut self, word: String, count: i32) {
        let word = if self.case_sensitive {
            word
        } else {
            word.to_lowercase()
        };
        *self.dictionary.entry(word).or_insert(0) += count;
    }
    fn update(&mut self) {
        self.total_words = self.dictionary.values().sum();
        self.unique_words = self.dictionary.len() as i32;
        self.longest_word = self.dictionary.keys().map(|word| word.len()).max().unwrap_or(0);
        self.letters = self.dictionary.keys().map(|word| word.chars().map(|c| c).collect::<Vec<char>>()).flatten().collect();
    }
}