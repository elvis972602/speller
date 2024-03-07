pub mod error;
pub mod source;
mod string;

use crate::error::BuildError;
#[cfg(feature = "csv")]
use crate::source::CsvWordLoader;
#[cfg(feature = "serde_json")]
use crate::source::JsonWordLoader;
use crate::source::{TextWordLoader, WordLoader};
use crate::string::PUNCTUATION;
use levenshtein_automata::{Distance, LevenshteinAutomatonBuilder};

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct Speller {
    distance: i32,
    case_sensitive: bool,
    automaton_builder: LevenshteinAutomatonBuilder,
    word_frequency: WordFrequency,
}

impl Speller {
    pub fn builder(local_file: Vec<String>) -> SpellerBuilder {
        SpellerBuilder::new(local_file)
    }

    fn known(&self, word: &str) -> bool {
        let word = if self.case_sensitive {
            word.to_string()
        } else {
            word.to_lowercase()
        };
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

    /// With same distance, return the word with the highest frequency. If multiple words have the same frequency, return first found (random).
    pub fn correction(&self, word: &str) -> Option<String> {
        if !self.should_check(word) {
            return None;
        }
        if self.known(word) {
            return Some(word.to_string());
        }
        let word = if self.case_sensitive {
            word.to_string()
        } else {
            word.to_lowercase()
        };

        let dfa = self.automaton_builder.build_dfa(&word);

        let mut best_match: Option<(String, u8)> = None;

        for item in self.word_frequency.list.iter() {
            match dfa.eval(item) {
                Distance::Exact(distance) if distance <= self.distance as u8 => {
                    // match exact word
                    if distance <= 1 {
                        return Some(item.to_string());
                    }
                    // add to best match
                    if best_match.as_ref().map_or(true, |&(_, d)| distance < d) {
                        best_match = Some((item.to_string(), distance));
                    }
                }
                _ => {}
            }
        }

        best_match.map(|(word, _)| word)
    }

    /// Return all possible candidates with the given distance.
    pub fn candidates(&self, word: &str, distance: u8) -> Option<Vec<Vec<String>>> {
        if !self.should_check(word) {
            return None;
        }
        let word = if self.case_sensitive {
            word.to_string()
        } else {
            word.to_lowercase()
        };

        let dfa = self.automaton_builder.build_dfa(&word);

        let mut matches: Vec<Vec<String>> = vec![Vec::new(); (distance as usize) + 1];

        if self.known(&word) {
            matches[0].push(word.to_string());
            return Some(matches);
        }

        let mut found = false;

        for item in self.word_frequency.list.iter() {
            match dfa.eval(item) {
                Distance::Exact(dist) if dist as usize <= distance as usize => {
                    matches[dist as usize].push(item.to_string());
                    found = true;
                }
                _ => {}
            }
        }

        if found {
            Some(matches)
        } else {
            None
        }
    }

    /// edit distance can only compute the exact Levenshtein distance up to a given `self.distance`.
    /// Over this distance, the automaton will invariably return `None`.
    pub fn edit_distance(&self, word1: &str, word2: &str) -> Option<u8> {
        let word1 = if self.case_sensitive {
            word1.to_string()
        } else {
            word1.to_lowercase()
        };
        let word2 = if self.case_sensitive {
            word2.to_string()
        } else {
            word2.to_lowercase()
        };
        let dfa = self.automaton_builder.build_dfa(&word1);
        match dfa.eval(word2) {
            Distance::Exact(distance) => Some(distance),
            _ => None,
        }
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
    local_file: Vec<String>,
    distance: i32,
    case_sensitive: bool,
    dict_source: Vec<HashMap<String, i32>>,
}

impl SpellerBuilder {
    pub fn new(local_file: Vec<String>) -> SpellerBuilder {
        SpellerBuilder {
            local_file,
            distance: 2,
            case_sensitive: false,
            dict_source: vec![],
        }
    }

    pub fn local_file(&mut self, local_file: Vec<String>) -> &mut Self {
        self.local_file = local_file;
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

    pub fn dict_source(&mut self, dict_source: Vec<HashMap<String, i32>>) -> &mut Self {
        self.dict_source = dict_source;
        self
    }

    pub fn build(&self) -> Result<Speller, BuildError> {
        if self.local_file.is_empty() {
            return Err(BuildError::DictFileNotFound);
        }

        let mut speller = Speller {
            distance: self.distance,
            case_sensitive: self.case_sensitive,
            automaton_builder: LevenshteinAutomatonBuilder::new(self.distance as u8, true),
            word_frequency: WordFrequency::new(self.case_sensitive),
        };

        for local_dictionary in self.local_file.iter() {
            let path = Path::new(local_dictionary);
            match path.extension().and_then(OsStr::to_str) {
                Some("json") => {
                    let mut loader = JsonWordLoader::default();
                    speller.word_frequency.load_file(&mut loader, path)?;
                }
                Some("csv") => {
                    let mut loader = CsvWordLoader::default();
                    speller.word_frequency.load_file(&mut loader, path)?;
                }
                Some("tsv") => {
                    let mut loader = CsvWordLoader::new().with_delimiter(b'\t');
                    speller.word_frequency.load_file(&mut loader, path)?;
                }
                Some("txt") => {
                    let mut loader = TextWordLoader::default();
                    speller.word_frequency.load_file(&mut loader, path)?;
                }
                _ => return Err(BuildError::FileTypeNotSupported),
            };
        }

        for dict in self.dict_source.iter() {
            speller.word_frequency.load_dict(dict.clone())?;
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

    pub fn load_file<W, P>(&mut self, source: &mut W, path: P) -> Result<(), BuildError>
    where
        W: WordLoader,
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data = source.load_words(reader)?;
        self.load_dict(data)?;
        Ok(())
    }

    pub fn load_dict(&mut self, dict: HashMap<String, i32>) -> Result<(), BuildError> {
        dict.iter().for_each(|(word, count)| {
            self.add_word(word.to_string(), *count);
        });
        self.update();
        Ok(())
    }

    fn add_word(&mut self, word: String, count: i32) {
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
        self.longest_word = self
            .dictionary
            .keys()
            .map(|word| word.len())
            .max()
            .unwrap_or(0);
    }
}
