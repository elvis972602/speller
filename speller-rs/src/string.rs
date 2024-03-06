use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static!(
    pub static ref PUNCTUATION: HashSet<char> = r#"!"$%&'()*+,-./:;<=>?@[\]^_`{|}~#"#.chars().collect();
);