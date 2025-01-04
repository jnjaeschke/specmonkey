use crate::SMResult;
use jwalk::WalkDir;
use std::path::Path;
use std::sync::Arc;

pub(crate) fn gather_files<P: AsRef<Path>>(
    directory: P,
    extensions: Arc<Vec<String>>,
) -> SMResult<Vec<String>> {
    let files = WalkDir::new(directory)
        .try_into_iter()
        .map_err(|e| e.into_io_error().unwrap())?
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
