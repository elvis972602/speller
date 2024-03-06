mod string;
mod error;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use levenshtein_automata::{Distance, LevenshteinAutomatonBuilder};
use crate::error::BuildError;
use crate::string::PUNCTUATION;

pub struct Speller {
    distance: i32,
    case_sensitive: bool,
    automaton_builder: LevenshteinAutomatonBuilder,
    word_frequency: WordFrequency,
}

impl Speller {
    pub fn builder() -> SpellerBuilder {
        SpellerBuilder::new()
    }

    fn known(&self, word: &str) -> bool {
        let word = if self.case_sensitive { word.to_string() } else { word.to_lowercase() };
        self.word_frequency.dictionary.contains_key(&word)
    }

    fn should_check(&self, word: &str) -> bool {
        let len = word.len();
        if len == 1 && PUNCTUATION.contains(&word.chars().next().unwrap()) {
            return false;
        }
        if len > self.word_frequency.longest_word + self.distance as usize {
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

    pub fn correction(&self, word: &str) -> Option<String> {
        if !self.should_check(word) {
            return None;
        }
        if self.known(word) {
            return Some(word.to_string());
        }
        let word = if self.case_sensitive { word.to_string() } else { word.to_lowercase() };

        let dfa = self.automaton_builder.build_dfa(&word);

        let mut d2 = None;

        for item in self.word_frequency.list.iter() {
            match dfa.eval(item) {
                Distance::Exact(0) |
                Distance::Exact(1) => {
                    return Some(item.to_string());
                }
                Distance::Exact(2) => {
                    if d2.is_none() {
                        d2 = Some(item.to_string());
                    }
                }
                _ => {}
            }
        }

        d2
    }

    pub fn languages() -> Vec<String> {
        // find all json files in data folder
        let paths = std::fs::read_dir("data").unwrap();
        let mut languages: Vec<String> = vec![];
        for path in paths {
            let path = path.unwrap().path();
            if path.extension() == Some(OsStr::new("json")) {
                let lang = path.file_stem().unwrap().to_str().unwrap().to_string();
                languages.push(lang);
            }
        }
        languages
    }
}

pub struct SpellerBuilder {
    language: Vec<String>,
    local_dictionary: Option<Vec<String>>,
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

    pub fn local_dictionary(&mut self, local_dictionary: Option<Vec<String>>) -> &mut Self {
        self.local_dictionary = local_dictionary;
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

    pub fn build(&self) -> Result<Speller, BuildError> {
        let mut speller = Speller {
            distance: self.distance,
            case_sensitive: self.case_sensitive,
            automaton_builder: LevenshteinAutomatonBuilder::new(self.distance as u8, true),
            word_frequency: WordFrequency::new(self.case_sensitive),
        };

        if let Some(local_dictionary) = &self.local_dictionary {
            for local_dictionary in local_dictionary {
                let path = Path::new(local_dictionary);
                if path.extension() == Some(OsStr::new("json")) {
                    speller.word_frequency.load_json(path)?;
                } else {
                    return Err(BuildError::NotJsonFile);
                }
            }
        }
        if self.language.len() > 0 {
            for lang in &self.language {
                let path = format!("data/{}.json", lang);
                speller.word_frequency.load_json(path)?;
            }
        }

        Ok(speller)
    }
}

pub struct WordFrequency {
    dictionary: HashMap<String, i32>,
    list: Vec<String>,
    unique_words: i32,
    case_sensitive: bool,
    longest_word: usize,
}

impl WordFrequency {
    pub fn new(case_sensitive: bool) -> WordFrequency {
        WordFrequency {
            dictionary: HashMap::new(),
            list: vec![],
            unique_words: 0,
            case_sensitive,
            longest_word: 0,
        }
    }

    pub fn load_json<T: AsRef<Path>>(&mut self, path: T) -> Result<(), BuildError> {
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
        let mut map_vec: Vec<(&String, &i32)> = self.dictionary.iter().collect();
        map_vec.sort_by_key(|a| a.1);
        map_vec.iter().for_each(|(word, _count)| {
            self.list.push(word.to_string());
        });
        self.unique_words = self.dictionary.len() as i32;
        self.longest_word = self.dictionary.keys().map(|word| word.len()).max().unwrap_or(0);
    }
}