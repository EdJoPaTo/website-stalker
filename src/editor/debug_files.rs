use std::path::Path;
use std::time::SystemTime;

use super::Content;
use crate::logger;

pub fn debug_files(path: &Path, content: Content) -> anyhow::Result<Content> {
    std::fs::create_dir_all(path)?;

    // Famous last words: Assume by good faith that time works and creates distinct filenames
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let mut filename = format!("{}-{}", timestamp.as_secs(), timestamp.subsec_nanos());
    if let Some(extension) = content.extension {
        filename += ".";
        filename += extension;
    }

    let file = path.join(filename);
    logger::warn(&format!("debug_files writes {}", file.display()));
    std::fs::write(file, &content.text)?;

    Ok(content)
}
