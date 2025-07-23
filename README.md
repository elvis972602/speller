# Speller

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org)
[![PyO3](https://img.shields.io/badge/PyO3-0.25.1-blue.svg)](https://github.com/PyO3/pyo3)

A high-performance spelling checker that uses [Levenshtein automaton](https://en.wikipedia.org/wiki/Levenshtein_automaton) to provide spell checking with **linear time complexity**. Built in Rust with Python bindings.

## Features

- ⚡ **Linear time complexity** using Levenshtein automaton
- 🔧 **Configurable edit distance** (1-3 characters)
- 📝 **Multiple dictionary formats** (JSON, CSV, TSV, TXT)
- 🔍 **Case-sensitive/insensitive** spell checking
- 🐍 **Python bindings** for easy integration
- 🚀 **High performance** with Rust implementation

## Quick Start

### Rust

Add to your `Cargo.toml`:
```toml
[dependencies]
speller-rs = { path = "speller-rs", features = ["serde_json", "csv"] }
```

```rust
use speller_rs::Speller;

// Create speller with JSON dictionary
let speller = Speller::builder()
    .dict_file(vec!["data/en.json".to_string()])
    .distance(2)
    .case_sensitive(false)
    .build()?;

// Get correction for misspelled word
if let Some(correction) = speller.correction("helo") {
    println!("Did you mean: {}", correction); // "hello"
}

// Get all candidates within distance
if let Some(candidates) = speller.candidates("helo", 2) {
    for (distance, words) in candidates.iter().enumerate() {
        println!("Distance {}: {:?}", distance, words);
    }
}
```

### Python

Install the Python package:
```bash
cd speller-py
maturin develop
```

```python
import speller_py

# Create speller instance
speller = speller_py.Speller(
    distance=2,
    case_sensitive=False,
    dict_file=['data/en.json']
)

# Get correction
correction = speller.correction("helo")
print(f"Did you mean: {correction}")  # "hello"

# Get all candidates
candidates = speller.candidates("helo", 2)
for distance, words in enumerate(candidates):
    if words:
        print(f"Distance {distance}: {words}")

# Calculate edit distance
distance = speller_py.edit_distance("hello", "helo", 10)
print(f"Edit distance: {distance}")  # 1
```

## Dictionary Formats

### JSON Format
```json
{
    "hello": 1000,
    "world": 800,
    "python": 500
}
```

### CSV Format
```csv
word,frequency
hello,1000
world,800
python,500
```

### TSV Format
```tsv
hello	1000
world	800
python	500
```

### TXT Format
```txt
hello 1000
world 800
python 500
```

## Directory Structure

For default language dictionaries, organize files as:
```
your_project/
├── src/
└── data/
    ├── en.json      # English dictionary
    ├── de.json      # German dictionary
    ├── fr.json      # French dictionary
    └── ...
```

## Configuration Options

### Rust API

```rust
let speller = Speller::builder()
    .dict_file(vec!["data/en.json".to_string()])  // Dictionary files
    .distance(2)                                   // Max edit distance (1-3)
    .case_sensitive(false)                        // Case sensitivity
    .build()?;
```

### Python API

```python
speller = speller_py.Speller(
    distance=2,              # Max edit distance
    case_sensitive=False,    # Case sensitivity
    dict_file=['data/en.json'],  # Dictionary files
    dict=[{'hello': 100, 'world': 200}]  # Or direct dictionary
)
```

## Advanced Usage

### Custom CSV Loading

```rust
use speller_rs::source::CsvWordLoader;

let mut loader = CsvWordLoader::new()
    .with_word_index(1)      // Word column index
    .with_count_index(2)     // Frequency column index
    .with_headers(true)      // Has header row
    .with_delimiter(b'\t');  // Tab separator

let speller = Speller::builder()
    .dict_file(vec!["custom.tsv".to_string()])
    .build()?;
```

### Multiple Dictionaries

```rust
let speller = Speller::builder()
    .dict_file(vec![
        "data/en.json".to_string(),
        "data/technical.csv".to_string(),
        "custom_words.txt".to_string(),
    ])
    .build()?;
```

## Performance

The Levenshtein automaton provides **O(n)** time complexity for spell checking, making it significantly faster than traditional approaches for large dictionaries.

Run benchmarks:
```bash
cargo bench
```

## Development

### Building
```bash
# Build Rust library
cargo build --features serde_json,csv

# Build Python bindings
cd speller-py && maturin develop
```

### Testing
```bash
# Run Rust tests
cargo test --features serde_json,csv

# Test with specific dictionary format
cargo test test_speller_json
```

## Architecture

This project is structured as a Cargo workspace:

- **`speller-rs/`**: Core Rust implementation
  - Levenshtein automaton-based spell checker
  - Multiple dictionary format support
  - High-performance edit distance calculation

- **`speller-py/`**: Python bindings using PyO3
  - Pythonic API wrapper
  - Exception handling
  - Type hints support

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`cargo test --features serde_json,csv`)
4. Commit your changes (`git commit -m 'Add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.