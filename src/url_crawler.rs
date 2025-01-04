use linkify::{LinkFinder, LinkKind};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{self, BufRead},
    sync::Arc,
};
use url::Url;

pub struct Link {
    pub(super) url: String,
    pub(super) domain: String,
    pub(super) filepath: String,
    pub(super) line_number: usize,
}

impl Link {
    fn new(arg_tuple: (String, String, usize)) -> Self {
        let (url_string, domain, line_number) = arg_tuple;
        Self {
            url: url_string,
            domain,
            line_number,
            filepath: Default::default(),
        }
    }
}

pub struct URLCrawler {
    filepaths: Vec<String>,
    whitelist_domains: Arc<Vec<String>>,
}

impl URLCrawler {
    fn new(filepaths: Vec<String>, whitelist_domains: Vec<String>) -> Self {
        let whitelist_lowercase: Arc<Vec<_>> = Arc::new(
            whitelist_domains
                .into_iter()
                .map(|url| url.to_lowercase())
                .collect(),
        );

        Self {
            filepaths,
            whitelist_domains: whitelist_lowercase,
        }
    }

    pub fn find_urls(filepaths: Vec<String>, whitelist_domains: Vec<String>) -> Vec<Link> {
        Self::new(filepaths, whitelist_domains).find()
    }

    fn find(&self) -> Vec<Link> {
        self.filepaths
            .par_iter()
            .filter_map(|filepath| {
                File::open(&filepath)
                    .ok()
                    .map(|file_pointer| (filepath, file_pointer))
            })
            .map(|(filepath, file_pointer)| self.find_urls_in_file(filepath, file_pointer))
            .flat_map(|urls_per_file| urls_per_file)
            .collect()
    }

    fn find_urls_in_file(&self, filepath: &String, file_pointer: File) -> Vec<Link> {
        let reader = io::BufReader::new(file_pointer);
        Self::find_urls_in_stream(reader)
            .into_iter()
            .filter_map(|(url_string, line_number)| self.filter_domains(url_string, line_number))
            .map(Link::new)
            .map(|mut link| {
                link.filepath = filepath.clone();
                link
            })
            .collect()
    }

    fn find_urls_in_stream<R: BufRead>(stream: R) -> Vec<(String, usize)> {
        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);

        stream
            .lines()
            .enumerate()
            .filter_map(|(line_number, line)| match line {
                Ok(l) => Some(
                    finder
                        .links(&l)
                        .map(|link| (link.as_str().to_string(), line_number + 1))
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .flatten()
            .collect()
    }

    fn filter_domains(
        &self,
        url_string: String,
        line_number: usize,
    ) -> Option<(String, String, usize)> {
        if let Ok(parsed_url) = Url::parse(&url_string) {
            if let Some(host) = parsed_url.host_str() {
                let host_lowercase = host.to_lowercase();
                let domains = self.whitelist_domains.clone();
                if domains.is_empty() {
                    return Some((url_string, host_lowercase, line_number));
                }
                return domains
                    .iter()
                    .find_map(|domain| {
                        if *domain == host_lowercase
                            || host_lowercase.ends_with(&format!(".{}", domain))
                        {
                            Some(domain.clone())
                        } else {
                            None
                        }
                    })
                    .map(|domain| (url_string, domain, line_number));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::rstest;
    use std::io::Cursor;

    #[test]
    fn test_extract_links_from_reader_empty() {
        let input = "";
        let reader = Cursor::new(input);
        let result = URLCrawler::find_urls_in_stream(reader);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_links_from_reader_single_url() {
        let input = "Visit https://example.com for more information.";
        let reader = Cursor::new(input);
        let result = URLCrawler::find_urls_in_stream(reader);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "https://example.com");
        assert_eq!(result[0].1, 1);
    }

    #[test]
    fn test_extract_links_from_reader_multiple_urls() {
        let input = "First URL: http://foo.specs.open.org/folder/#section-2.\nSecond URL: https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo.";
        let reader = Cursor::new(input);
        let result = URLCrawler::find_urls_in_stream(reader);
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
        let result = URLCrawler::find_urls_in_stream(reader);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_links_from_reader_various_urls() {
        let input = "Check out https://example.com/#section-1 and some text.\nAnother link: http://foo.specs.open.org/folder/#section-2.\nEnd with URL https://bugzil.la/5678#:~:text=foo-,bar%20baz,-blah.";
        let reader = Cursor::new(input);
        let result = URLCrawler::find_urls_in_stream(reader);
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

    #[rstest]
    #[case("https://example.com", true)]
    #[case("https://no-example.com", false)]
    #[case("https://subdomain.example.com", true)]
    #[case("https://example.com/foo", true)]
    #[case("http://example.com", true)]
    fn test_filter_match(#[case] url: &str, #[case] should_match: bool) {
        let domains = ["example.com"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();
        let crawler = URLCrawler::new(vec![], domains);
        assert_eq!(
            crawler.filter_domains(String::from(url), 0).is_some(),
            should_match
        );
    }
}
