use crate::error::BuildError;
use std::collections::HashMap;
use std::io;
use std::io::BufRead;

pub trait WordLoader {
    fn load_words<R: io::Read>(&mut self, reader: R) -> Result<HashMap<String, i32>, BuildError>;
}

#[cfg(feature = "serde_json")]
pub struct JsonWordLoader {}

#[cfg(feature = "serde_json")]
impl JsonWordLoader {
    pub fn new() -> JsonWordLoader {
        JsonWordLoader {}
    }
}

impl WordLoader for JsonWordLoader {
    fn load_words<R: io::Read>(&mut self, reader: R) -> Result<HashMap<String, i32>, BuildError> {
        let data: HashMap<String, i32> = serde_json::from_reader(reader)?;
        Ok(data)
    }
}

impl Default for JsonWordLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "csv")]
pub struct CsvWordLoader {
    word_index: usize,
    count_index: usize,
    delimiter: u8,
    has_headers: bool,
}

#[cfg(feature = "csv")]
impl CsvWordLoader {
    pub fn new() -> CsvWordLoader {
        CsvWordLoader {
            word_index: 0,
            count_index: 1,
            delimiter: b',',
            has_headers: false,
        }
    }

    pub fn with_word_index(mut self, index: usize) -> CsvWordLoader {
        self.word_index = index;
        self
    }

    pub fn with_count_index(mut self, index: usize) -> CsvWordLoader {
        self.count_index = index;
        self
    }

    pub fn with_delimiter(mut self, delimiter: u8) -> CsvWordLoader {
        self.delimiter = delimiter;
        self
    }

    pub fn with_headers(mut self, has_headers: bool) -> CsvWordLoader {
        self.has_headers = has_headers;
        self
    }
}

impl Default for CsvWordLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "csv")]
impl WordLoader for CsvWordLoader {
    fn load_words<R: io::Read>(&mut self, reader: R) -> Result<HashMap<String, i32>, BuildError> {
        let mut data = HashMap::new();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .from_reader(reader);
        for result in rdr.records() {
            let record = result?;
            let word = record
                .get(self.word_index)
                .ok_or(BuildError::CSVIndexError)?;
            let count = record
                .get(self.count_index)
                .ok_or(BuildError::CSVIndexError)?;
            data.insert(
                word.to_string(),
                count.parse().map_err(|_| BuildError::ParseCountError)?,
            );
        }
        Ok(data)
    }
}

pub struct TextWordLoader {
    word_index: usize,
    count_index: usize,
    delimiter: u8,
    has_headers: bool,
}

impl TextWordLoader {
    pub fn new() -> TextWordLoader {
        TextWordLoader {
            word_index: 0,
            count_index: 1,
            delimiter: b' ',
            has_headers: false,
        }
    }

    pub fn with_word_index(mut self, index: usize) -> TextWordLoader {
        self.word_index = index;
        self
    }

    pub fn with_count_index(mut self, index: usize) -> TextWordLoader {
        self.count_index = index;
        self
    }

    pub fn with_delimiter(mut self, delimiter: u8) -> TextWordLoader {
        self.delimiter = delimiter;
        self
    }

    pub fn with_headers(mut self, has_headers: bool) -> TextWordLoader {
        self.has_headers = has_headers;
        self
    }
}

impl Default for TextWordLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl WordLoader for TextWordLoader {
    fn load_words<R: io::Read>(&mut self, reader: R) -> Result<HashMap<String, i32>, BuildError> {
        let mut data = HashMap::new();
        let reader = io::BufReader::new(reader);
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(self.delimiter as char).collect();
            let word = parts
                .get(self.word_index)
                .ok_or(BuildError::TXTIndexError)?;
            let count = parts
                .get(self.count_index)
                .ok_or(BuildError::TXTIndexError)?;
            data.insert(
                word.to_string(),
                count.parse().map_err(|_| BuildError::ParseCountError)?,
            );
        }
        Ok(data)
    }
}
