use lazy_static::lazy_static;
use std::collections::HashSet;

#[rustfmt::skip]
lazy_static!(
    pub static ref PUNCTUATION: HashSet<char> =
    r#"!"$%&'()*+,-./:;<=>?@[\]^_`{|}~#"#.chars().collect();
);
