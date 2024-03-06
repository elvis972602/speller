use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use ::speller_rs as speller_rs;

#[pyclass]
struct Speller(speller_rs::Speller);

#[pymethods]
impl Speller {
    #[new]
    #[pyo3(text_signature = "(language, distance=2, case_sensitive=False, local_file=None)")]
    fn new(language: Vec<String>, distance: i32, case_sensitive: bool, local_file: Option<Vec<String>>) -> PyResult<Self> {
        let speller = speller_rs::Speller::builder()
            .language(language)
            .local_dictionary(local_file)
            .distance(distance)
            .case_sensitive(case_sensitive)
            .build();
        let speller = speller.map_err(|e| BuildError::new_err(e.to_string()))?;
        Ok(Speller(speller))
    }

    #[pyo3(text_signature = "($self, input)")]
    fn correction(&self, word: &str) -> PyResult<Option<String>> {
        Ok(self.0.correction(word))
    }
}

#[pyfunction]
fn languages() -> PyResult<Vec<String>> {
    Ok(speller_rs::Speller::languages())
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
    m.add_function(wrap_pyfunction!(languages, m)?)?;
    Ok(())
}