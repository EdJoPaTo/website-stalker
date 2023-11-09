use std::fs::{create_dir_all, read_dir, read_to_string, remove_file, write};
use std::path::{Path, PathBuf};

use crate::ChangeKind;

/// Remove site files which are no longer configured to cleanup the directory
pub fn remove_gone(expected_paths: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    fn inner(expected_paths: &[PathBuf], path: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut superfluous = Vec::new();
        for entry in read_dir(path)? {
            let entry = entry?.path();
            if entry.is_dir() {
                superfluous.append(&mut inner(expected_paths, &entry)?);
            } else {
                // Expected is without extension
                let is_wanted = expected_paths.contains(&entry.with_extension(""));
                if !is_wanted {
                    remove_file(&entry)?;
                    superfluous.push(entry);
                }
            }
        }
        Ok(superfluous)
    }

    let mut superfluous = Vec::new();
    for entry in read_dir(".")? {
        let entry = entry?.path();
        if !entry.is_dir() {
            continue;
        }
        if let Some(filename) = entry.file_name() {
            let is_relevant = filename.to_str().map_or(false, |o| !o.starts_with('.'));
            if is_relevant {
                superfluous.append(&mut inner(expected_paths, Path::new(filename))?);
            }
        }
    }
    superfluous.sort();
    Ok(superfluous)
}

/// Remove files with the same base but a different extension.
/// This cleans up changes of the extension like `html` -> `md`.
fn remove_same_base_different_extension(path: &Path) -> anyhow::Result<bool> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem();
    let extension = path.extension();
    let mut removed_something = false;
    for file in read_dir(parent)? {
        let file = file?.path();

        let same_stem = file.file_stem() == stem;
        if !same_stem {
            continue;
        }

        let same_extension = file.extension() == extension;
        let remove = !same_extension;

        if remove {
            remove_file(file)?;
            removed_something = true;
        }
    }
    Ok(removed_something)
}

pub fn write_only_changed(path: &Path, content: &str) -> anyhow::Result<ChangeKind> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let content = content.trim().to_owned() + "\n";

    let current = read_to_string(path).unwrap_or_default();
    let changed = current != content;
    if changed {
        write(path, content)?;
    }

    let removed_something = remove_same_base_different_extension(path)?;

    if removed_something {
        Ok(ChangeKind::Changed)
    } else if current.is_empty() {
        Ok(ChangeKind::Init)
    } else if changed {
        Ok(ChangeKind::Changed)
    } else {
        Ok(ChangeKind::ContentSame)
    }
}
