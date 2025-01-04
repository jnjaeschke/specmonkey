use crate::{error::SpecMonkeyError, url_crawler::Link, SMResult};
use std::{collections::HashMap, fs, path::Path};

pub struct Index {
    index: HashMap<String, HashMap<String, Vec<Link>>>,
}

impl Index {
    pub fn from_raw_data(raw_data: Vec<Link>) -> Self {
        let mut index = HashMap::new();
        for raw_url in raw_data {
            let fragment = {
                let parts: Vec<_> = raw_url.url.split("#").collect();
                if parts.len() > 1 {
                    parts.last().map(|fragment| fragment.to_string())
                } else {
                    None
                }
            }
            .unwrap_or_default();
            index
                .entry(raw_url.domain.clone())
                .or_insert_with(HashMap::new)
                .entry(fragment)
                .or_insert_with(Vec::new)
                .push(raw_url)
        }
        Self { index }
    }

    pub fn write_json<P: AsRef<Path>>(&self, output_dir: P) -> SMResult<()> {
        if output_dir.as_ref().exists() && !output_dir.as_ref().is_dir() {
            return Err(SpecMonkeyError::Error(String::from(
                "Output directory must be a directory.",
            )));
        }
        fs::create_dir_all(&output_dir)?;
        for (domain, items) in &self.index {
            let filename = format!("{}.json", domain);
            let filepath = output_dir.as_ref().join(filename);

            // Open the file in write mode, creating it if it doesn't exist.
            let file = fs::File::create(&filepath)?;

            // Serialize the Vec<IndexItem> to JSON and write it to the file.
            serde_json::to_writer_pretty(file, items)?;
        }
        Ok(())
    }
}
