use ::speller_rs;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
struct Speller(speller_rs::Speller);

#[pymethods]
impl Speller {
    #[new]
    #[pyo3(text_signature = "(distance, case_sensitive=False, dict_file=None, dict=None)")]
    fn new(
        distance: i32,
        case_sensitive: bool,
        dict_file: Option<Vec<String>>,
        dict: Option<Vec<HashMap<String, i32>>>,
    ) -> PyResult<Self> {
        let mut speller_builder = speller_rs::Speller::builder();
        speller_builder
            .distance(distance)
            .case_sensitive(case_sensitive);
        if let Some(dict_file) = dict_file {
            speller_builder.dict_file(dict_file);
        }
        if let Some(dict) = dict {
            speller_builder.dict_source(dict);
        }
        let speller = speller_builder
            .build()
            .map_err(|e| BuildError::new_err(e.to_string()))?;
        Ok(Speller(speller))
    }

    #[pyo3(text_signature = "($self, word)")]
    fn correction(&self, word: &str) -> PyResult<Option<String>> {
        Ok(self.0.correction(word))
    }

    #[pyo3(text_signature = "($self, word, distance)")]
    fn candidates(&self, word: &str, distance: u8) -> PyResult<Option<Vec<Vec<String>>>> {
        Ok(self.0.candidates(word, distance))
    }
}

#[pyfunction]
#[pyo3(text_signature = "(word1, word2, limit)")]
fn edit_distance(word1: &str, word2: &str, limit: usize) -> PyResult<Option<usize>> {
    Ok(speller_rs::edit_distance(word1, word2, limit))
}

create_exception!(
    speller_module,
    BuildError,
    PyException,
    "An error occurred while building the speller"
);

#[pymodule]
fn speller_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Speller>()?;
    m.add("BuildError", _py.get_type::<BuildError>())?;
    m.add_function(wrap_pyfunction!(edit_distance, m)?)?;
    Ok(())
}
