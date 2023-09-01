use std::fs::{create_dir_all, read_dir, read_to_string, remove_file, write};
use std::path::{Path, PathBuf};

use crate::ChangeKind;

const SITE_FOLDER: &str = "sites";

pub fn remove_gone(expected_basenames: &[String]) -> anyhow::Result<Vec<PathBuf>> {
    create_dir_all(SITE_FOLDER)?;
    let mut superfluous = Vec::new();
    for file in read_dir(SITE_FOLDER)? {
        let file = file?.path();
        let is_wanted = file
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .map_or(false, |name| basename_is_wanted(expected_basenames, name));
        if !is_wanted {
            remove_file(&file)?;
            superfluous.push(file);
        }
    }
    superfluous.sort();
    Ok(superfluous)
}

fn remove_same_base_different_extension(basename: &str, extension: &str) -> anyhow::Result<bool> {
    let mut removed_something = false;
    for file in read_dir(SITE_FOLDER)? {
        let file = file?.path();

        let same_stem = file
            .file_stem()
            .map_or(false, |o| o.to_str() == Some(basename));
        if !same_stem {
            continue;
        }

        let same_extension = file
            .extension()
            .map_or(false, |o| o.to_str() == Some(extension));
        let remove = !same_extension;

        if remove {
            remove_file(file)?;
            removed_something = true;
        }
    }
    Ok(removed_something)
}

pub fn write_only_changed(
    basename: &str,
    extension: &str,
    content: &str,
) -> anyhow::Result<ChangeKind> {
    create_dir_all(SITE_FOLDER)?;
    let path = Path::new(SITE_FOLDER).join(format!("{basename}.{extension}"));
    let content = content.trim().to_string() + "\n";

    let current = read_to_string(&path).unwrap_or_default();
    let changed = current != content;
    if changed {
        write(&path, content)?;
    }

    let removed_something = remove_same_base_different_extension(basename, extension)?;

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

fn basename_is_wanted(basenames: &[String], searched: &str) -> bool {
    for basename in basenames {
        if searched.starts_with(basename) {
            return true;
        }
    }
    false
}
