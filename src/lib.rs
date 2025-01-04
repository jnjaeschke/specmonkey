use pyo3::prelude::*;
use pyo3::types::PyModule;

mod url_crawler;
use url_crawler::{Link, URLCrawler};
/// A Python class representing a URL, its corresponding line number, and the file it was found in.
#[pyclass(name = "Link")]
struct PyLink {
    #[pyo3(get)]
    url: String,
    #[pyo3(get)]
    domain: String,
    #[pyo3(get)]
    filepath: String,
    #[pyo3(get)]
    line_number: usize,
}

#[pymethods]
impl PyLink {
    #[new]
    fn new(url: String, domain: String, filepath: String, line_number: usize) -> Self {
        PyLink {
            url,
            domain,
            filepath,
            line_number,
        }
    }
}

impl From<Link> for PyLink {
    fn from(value: Link) -> Self {
        let Link {
            url,
            domain,
            filepath,
            line_number,
        } = value;
        Self::new(url, domain, filepath, line_number)
    }
}

#[pyfunction]
fn extract_links(filepaths: Vec<String>, whitelist_domains: Vec<String>) -> PyResult<Vec<PyLink>> {
    Ok(URLCrawler::find_urls(filepaths, whitelist_domains)
        .into_iter()
        .map(PyLink::from)
        .collect())
}

/// A Python module implemented in Rust.
#[pymodule]
fn specmonkey_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLink>()?;
    m.add_function(wrap_pyfunction!(extract_links, m)?)?;
    Ok(())
}
