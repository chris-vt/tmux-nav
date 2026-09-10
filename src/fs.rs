use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FsItem {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub is_parent_link: bool,
}

impl FsItem {
    pub fn name(&self) -> String {
        if self.is_parent_link {
            return "..".to_string();
        }
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }
}

pub fn read_dir(path: &Path, depth: usize) -> io::Result<Vec<FsItem>> {
    let mut items = Vec::new();
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    items.push(FsItem {
                        path: entry.path(),
                        depth,
                        is_dir,
                        is_expanded: false,
                        is_parent_link: false,
                    });
                }
            }
        }
    }

    // Sort: directories first, then alphabetically
    items.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name().to_lowercase().cmp(&b.name().to_lowercase()))
    });

    Ok(items)
}
