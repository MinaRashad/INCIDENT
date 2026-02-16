/**
 * Tools and functions that are so common between scripts
 */
use std::path::PathBuf;
use crate::views::docs::DOCS_ROOT;

pub fn parent(path: PathBuf)-> PathBuf{
    match path.parent()
    .map(|p| p.to_path_buf()){
        Some(parent) => parent,
        None => PathBuf::from(DOCS_ROOT)
    }
}

