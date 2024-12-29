use linkify::{LinkFinder, LinkKind};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::Mutex;

/// A Python class representing a URL, its corresponding line number, and the file it was found in.
#[pyclass]
struct Link {
    #[pyo3(get)]
    url: String,
    #[pyo3(get)]
    line_number: usize,
    #[pyo3(get)]
    file_name: String,
}

#[pymethods]
impl Link {
    #[new]
    fn new(url: String, line_number: usize, file_name: String) -> Self {
        Link {
            url,
            line_number,
            file_name,
        }
    }
}

/// Extracts all URLs from a given input stream.
/// This function is agnostic of the input source (file, network stream, etc.).
/// It reads the input line by line and returns a vector of (URL, line_number) tuples.
fn extract_links_from_reader<R: BufRead>(reader: R) -> Vec<(String, usize)> {
    let mut links = Vec::new();
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    for (line_number, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            for link in finder.links(&line) {
                links.push((link.as_str().to_string(), line_number + 1));
            }
        }
    }

    links
}

/// Extracts all URLs from the given list of file paths.
/// Processes files in parallel for efficiency.
/// Returns a list of Link objects containing the URL, line number, and file name.
#[pyfunction]
fn extract_links(file_paths: Vec<String>) -> PyResult<Vec<Link>> {
    // Initialize a thread-safe vector to collect all links
    let links = Mutex::new(Vec::new());

    // Process each file in parallel using Rayon
    file_paths.par_iter().for_each(|file_path| {
        // Attempt to open the file
        if let Ok(file) = File::open(file_path) {
            let reader = io::BufReader::new(file);
            // Extract links using the separate function
            let extracted_links = extract_links_from_reader(reader);
            // Create Link instances and add them to the shared vector
            for (url, line_number) in extracted_links {
                let link_entry = Link::new(url, line_number, file_path.clone());
                links.lock().unwrap().push(link_entry);
            }
        }
        // If the file cannot be opened, it is silently skipped.
        // You can modify this behavior to log errors or handle them as needed.
    });

    // Retrieve the collected links from the mutex
    let collected_links = links.into_inner().unwrap();
    Ok(collected_links)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extract_links_from_reader_empty() {
        let input = "";
        let reader = Cursor::new(input);
        let result = extract_links_from_reader(reader);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_links_from_reader_single_url() {
        let input = "Visit https://example.com for more information.";
        let reader = Cursor::new(input);
        let result = extract_links_from_reader(reader);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "https://example.com");
        assert_eq!(result[0].1, 1);
    }

    #[test]
    fn test_extract_links_from_reader_multiple_urls() {
        let input = "First URL: http://foo.specs.open.org/folder/#section-2.\nSecond URL: https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo.";
        let reader = Cursor::new(input);
        let result = extract_links_from_reader(reader);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "http://foo.specs.open.org/folder/#section-2");
        assert_eq!(result[0].1, 1);
        assert_eq!(
            result[1].0,
            "https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo"
        );
        assert_eq!(result[1].1, 2);
    }

    #[test]
    fn test_extract_links_from_reader_no_urls() {
        let input = "This line has no links.\nNeither does this one.";
        let reader = Cursor::new(input);
        let result = extract_links_from_reader(reader);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_links_from_reader_various_urls() {
        let input = "Check out https://example.com/#section-1 and some text.\nAnother link: http://foo.specs.open.org/folder/#section-2.\nEnd with URL https://bugzil.la/5678#:~:text=foo-,bar%20baz,-blah.";
        let reader = Cursor::new(input);
        let result = extract_links_from_reader(reader);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "https://example.com/#section-1");
        assert_eq!(result[0].1, 1);
        assert_eq!(result[1].0, "http://foo.specs.open.org/folder/#section-2");
        assert_eq!(result[1].1, 2);
        assert_eq!(
            result[2].0,
            "https://bugzil.la/5678#:~:text=foo-,bar%20baz,-blah"
        );
        assert_eq!(result[2].1, 3);
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn specmonkey_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Link>()?;
    m.add_function(wrap_pyfunction!(extract_links, m)?)?;
    Ok(())
}
