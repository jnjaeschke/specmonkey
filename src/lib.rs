use std::io;
use std::sync::Arc;

use jwalk::WalkDir;
use pyo3::types::PyModule;
use pyo3::{exceptions::PyIOError, prelude::*};

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

fn gather_files(directory: &str, extensions: Arc<Vec<String>>) -> Result<Vec<String>, io::Error> {
    let files = WalkDir::new(directory)
        .try_into_iter()?
        .filter_map(|p| p.ok())
        .filter(|p| p.path().is_file())
        .filter(|p| {
            let ext = extensions.clone();
            ext.is_empty()
                || ext.iter().any(|e| {
                    p.path()
                        .extension()
                        .unwrap_or_default()
                        .eq_ignore_ascii_case(e)
                })
        })
        .map(|p| String::from(p.path().to_str().unwrap_or_default()))
        .collect::<Vec<_>>();
    Ok(files)
}

#[pyfunction]
fn extract_links(
    directory: String,
    extensions: Vec<String>,
    whitelist_domains: Vec<String>,
) -> PyResult<Vec<PyLink>> {
    gather_files(&directory, Arc::new(extensions))
        .and_then(|filepaths| {
            Ok(URLCrawler::find_urls(filepaths, whitelist_domains)
                .into_iter()
                .map(PyLink::from)
                .collect())
        })
        .map_err(|err| PyIOError::new_err(err.to_string()))
}

/// A Python module implemented in Rust.
#[pymodule]
fn specmonkey_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLink>()?;
    m.add_function(wrap_pyfunction!(extract_links, m)?)?;
    Ok(())
}
