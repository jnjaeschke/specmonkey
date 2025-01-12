use crate::SMResult;
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn gather_files<P: AsRef<Path>>(
    directory: P,
    extensions: Arc<Vec<String>>,
    exclude_folders: Arc<Vec<PathBuf>>,
) -> SMResult<Vec<PathBuf>> {
    let files = WalkDir::new(&directory)
        .try_into_iter()
        .map_err(|e| e.into_io_error().unwrap())?
        .filter_map(|p| p.ok())
        .filter(|p| p.path().is_file())
        .filter(|p| {
            let relpath = p
                .path()
                .strip_prefix(&directory)
                .expect("How can this file not have its root as root?")
                .to_path_buf();
            !exclude_folders
                .iter()
                .any(|excl| relpath.starts_with(&excl))
        })
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
        .map(|p| p.path())
        .collect::<Vec<_>>();
    Ok(files)
}
