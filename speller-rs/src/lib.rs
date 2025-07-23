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
use std::{cmp, mem};

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
    pub fn builder() -> SpellerBuilder {
        SpellerBuilder::new()
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
                    if best_match.as_ref().is_none_or(|&(_, d)| distance < d) {
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
}

/// edit distance can only compute the exact Levenshtein distance up to a given `limit`.
/// Over this distance, it will invariably return `None`.
///
/// Reference: https://doc.rust-lang.org/beta/nightly-rustc/rustc_span/edit_distance/index.html
pub fn edit_distance(a: &str, b: &str, limit: usize) -> Option<usize> {
    let mut a = &a.chars().collect::<Vec<_>>()[..];
    let mut b = &b.chars().collect::<Vec<_>>()[..];

    // Ensure that `b` is the shorter string, minimizing memory use.
    if a.len() < b.len() {
        mem::swap(&mut a, &mut b);
    }

    let min_dist = a.len() - b.len();
    // If we know the limit will be exceeded, we can return early.
    if min_dist > limit {
        return None;
    }

    // Strip common prefix.
    while let Some((b_char, b_rest)) = b.split_first() {
        if let Some((a_char, a_rest)) = a.split_first() {
            if a_char == b_char {
                a = a_rest;
                b = b_rest;
                continue;
            }
        }
        break;
    }
    // Strip common suffix.
    while let Some((b_char, b_rest)) = b.split_last() {
        if let Some((a_char, a_rest)) = a.split_last() {
            if a_char == b_char {
                a = a_rest;
                b = b_rest;
                continue;
            }
        }
        break;
    }

    #[allow(clippy::len_zero)]
    // If either string is empty, the distance is the length of the other.
    // We know that `b` is the shorter string, so we don't need to check `a`.
    if b.len() == 0 {
        return Some(min_dist);
    }

    let mut prev_prev = vec![usize::MAX; b.len() + 1];
    let mut prev = (0..=b.len()).collect::<Vec<_>>();
    let mut current = vec![0; b.len() + 1];

    // row by row
    for i in 1..=a.len() {
        current[0] = i;
        let a_idx = i - 1;

        // column by column
        for j in 1..=b.len() {
            let b_idx = j - 1;

            // There is no cost to substitute a character with itself.
            let substitution_cost = if a[a_idx] == b[b_idx] { 0 } else { 1 };

            current[j] = cmp::min(
                // deletion
                prev[j] + 1,
                cmp::min(
                    // insertion
                    current[j - 1] + 1,
                    // substitution
                    prev[j - 1] + substitution_cost,
                ),
            );

            if (i > 1) && (j > 1) && (a[a_idx] == b[b_idx - 1]) && (a[a_idx - 1] == b[b_idx]) {
                // transposition
                current[j] = cmp::min(current[j], prev_prev[j - 2] + 1);
            }
        }

        // Rotate the buffers, reusing the memory.
        [prev_prev, prev, current] = [prev, current, prev_prev];
    }

    // `prev` because we already rotated the buffers.
    let distance = prev[b.len()];
    (distance <= limit).then_some(distance)
}

pub struct SpellerBuilder {
    dict_file: Vec<String>,
    distance: i32,
    case_sensitive: bool,
    dict_source: Vec<HashMap<String, i32>>,
}

impl Default for SpellerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SpellerBuilder {
    pub fn new() -> SpellerBuilder {
        SpellerBuilder {
            dict_file: vec![],
            distance: 2,
            case_sensitive: false,
            dict_source: vec![],
        }
    }

    pub fn dict_file(&mut self, local_file: Vec<String>) -> &mut Self {
        self.dict_file = local_file;
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
        let mut speller = Speller {
            distance: self.distance,
            case_sensitive: self.case_sensitive,
            automaton_builder: LevenshteinAutomatonBuilder::new(self.distance as u8, true),
            word_frequency: WordFrequency::new(self.case_sensitive),
        };

        for local_dictionary in self.dict_file.iter() {
            let path = Path::new(local_dictionary);
            match path.extension().and_then(OsStr::to_str) {
                #[cfg(feature = "serde_json")]
                Some("json") => {
                    let mut loader = JsonWordLoader::default();
                    speller.word_frequency.load_file(&mut loader, path)?;
                }
                #[cfg(feature = "csv")]
                Some("csv") => {
                    let mut loader = CsvWordLoader::default();
                    speller.word_frequency.load_file(&mut loader, path)?;
                }
                #[cfg(feature = "csv")]
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

        if speller.word_frequency.unique_words == 0 {
            return Err(BuildError::DictNotFound);
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
