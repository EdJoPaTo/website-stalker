use std::ffi::OsString;
use std::fs::{create_dir_all, read_dir, read_to_string, remove_file, write};

use crate::ChangeKind;

#[derive(Clone)]
pub struct SiteStore {
    folder: String,
}

impl SiteStore {
    pub fn new(folder: String) -> std::io::Result<Self> {
        create_dir_all(&folder)?;
        Ok(Self { folder })
    }

    pub fn remove_gone(&self, expected_basenames: &[String]) -> anyhow::Result<Vec<OsString>> {
        let mut superfluous = Vec::new();
        for file in read_dir(&self.folder)? {
            let file = file?;
            let is_wanted = file
                .file_name()
                .into_string()
                .map_or(false, |name| basename_is_wanted(expected_basenames, &name));
            if !is_wanted {
                remove_file(file.path())?;
                superfluous.push(file.file_name());
            }
        }
        superfluous.sort();
        Ok(superfluous)
    }

    pub fn remove_same_base_different_extension(
        &self,
        basename: &str,
        extension: &str,
    ) -> anyhow::Result<bool> {
        let mut removed_something = false;
        for file in read_dir(&self.folder)? {
            let file = file?;
            let remove = file.file_name().into_string().map_or(false, |name| {
                !name.ends_with(extension) && name.starts_with(&format!("{}.", basename))
            });
            if remove {
                remove_file(file.path())?;
                removed_something = true;
            }
        }
        Ok(removed_something)
    }

    pub fn write_only_changed(
        &self,
        basename: &str,
        extension: &str,
        content: &str,
    ) -> anyhow::Result<ChangeKind> {
        let path = format!("{}/{}.{}", self.folder, basename, extension);
        let content = content.trim().to_string() + "\n";

        let current = read_to_string(&path).unwrap_or_default();
        let changed = current != content;
        if changed {
            write(&path, content)?;
        }

        let removed_something = self.remove_same_base_different_extension(basename, extension)?;

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
}

fn basename_is_wanted(basenames: &[String], searched: &str) -> bool {
    for basename in basenames {
        if searched.starts_with(&format!("{}.", basename)) {
            return true;
        }
    }
    false
}
