#[cfg(test)]
#[cfg(feature = "serde_json")]
#[cfg(feature = "csv")]
mod test_loader {
    use speller_rs::source::{CsvWordLoader, JsonWordLoader, TextWordLoader, WordLoader};
    use std::fs::File;
    use std::io::BufReader;
    #[test]
    fn test_json() {
        let mut json_loader = JsonWordLoader::new();
        let mut reader = BufReader::new(File::open("./tests/files/en.json").unwrap());
        json_loader.load_words(&mut reader).unwrap();
    }

    mod test_csv {
        use super::*;

        #[test]
        fn test_csv() {
            let mut csv_loader = CsvWordLoader::new();
            let mut reader = BufReader::new(File::open("./tests/files/en.csv").unwrap());
            csv_loader.load_words(&mut reader).unwrap();
        }

        #[test]
        fn test_csv_with_headers() {
            let mut csv_loader = CsvWordLoader::new().with_headers(true);
            let mut reader =
                BufReader::new(File::open("./tests/files/en_with_header.csv").unwrap());
            csv_loader.load_words(&mut reader).unwrap();
        }

        #[test]
        fn test_csv_with_index() {
            let mut csv_loader = CsvWordLoader::new().with_word_index(1).with_count_index(2);
            let mut reader = BufReader::new(File::open("./tests/files/en_with_index.csv").unwrap());
            csv_loader.load_words(&mut reader).unwrap();
        }

        #[test]
        fn test_csv_with_index_and_headers() {
            let mut csv_loader = CsvWordLoader::new()
                .with_word_index(1)
                .with_count_index(2)
                .with_headers(true);
            let mut reader =
                BufReader::new(File::open("./tests/files/en_with_index_header.csv").unwrap());
            csv_loader.load_words(&mut reader).unwrap();
        }
    }

    #[test]
    fn test_tsv() {
        let mut csv_loader = CsvWordLoader::new().with_delimiter(b'\t');
        let mut reader = BufReader::new(File::open("./tests/files/en.tsv").unwrap());
        csv_loader.load_words(&mut reader).unwrap();
    }

    #[test]
    fn test_txt() {
        let mut txt_loader = TextWordLoader::new();
        let mut reader = BufReader::new(File::open("./tests/files/en.txt").unwrap());
        txt_loader.load_words(&mut reader).unwrap();
    }
}

#[cfg(test)]
#[cfg(feature = "serde_json")]
#[cfg(feature = "csv")]
mod test_speller {
    use speller_rs::Speller;

    #[test]
    fn test_speller_json() {
        let speller = Speller::builder(vec!["./tests/files/en.json".to_string()])
            .build()
            .unwrap();
        let words = ["Yessss", "conticorrantue", "obrigada", "oi_biagomes", "haa"];
        for word in words.iter() {
            let _correct = speller.correction(word);
            let _candidates = speller.candidates(word, 2);
        }
    }

    #[test]
    fn test_speller_csv() {
        let speller = Speller::builder(vec!["./tests/files/en.csv".to_string()])
            .build()
            .unwrap();
        let words = ["Yessss", "conticorrantue", "obrigada", "oi_biagomes", "haa"];
        for word in words.iter() {
            let _correct = speller.correction(word);
            let _candidates = speller.candidates(word, 2);
        }
    }

    #[test]
    fn test_speller_tsv() {
        let speller = Speller::builder(vec!["./tests/files/en.tsv".to_string()])
            .build()
            .unwrap();
        let words = ["Yessss", "conticorrantue", "obrigada", "oi_biagomes", "haa"];
        for word in words.iter() {
            let _correct = speller.correction(word);
            let _candidates = speller.candidates(word, 2);
        }
    }

    #[test]
    fn test_speller_txt() {
        let speller = Speller::builder(vec!["./tests/files/en.txt".to_string()])
            .build()
            .unwrap();
        let words = ["Yessss", "conticorrantue", "obrigada", "oi_biagomes", "haa"];
        for word in words.iter() {
            let _correct = speller.correction(word);
            let _candidates = speller.candidates(word, 2);
        }
    }
}
