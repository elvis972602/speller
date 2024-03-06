# Speller: Levenshtein Automaton-Based Spelling Checker

Speller is a spelling checker that uses [Levenshtein automaton](https://en.wikipedia.org/wiki/Levenshtein_automaton) to
provides spell checking of linear time complexity.

## Note

If using default language, ensure to have dictionary files in ./data directory. The dictionary files should be in JSON format.

### Directory Structure

```plaintext
your_project/
├── other_stuff/
└── data/
    ├── en.json
    └── ...
```

## speller-py: Python Bindings for Speller

`speller-py` is a Python bindings for Speller. It provides a Python interface to the Speller library.