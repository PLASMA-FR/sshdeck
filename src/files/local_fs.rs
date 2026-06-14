use std::{
    fs,
    path::{Path, PathBuf},
};

use super::file_entry::{kind_from_name, FileEntry, FileKind};

pub fn list_dir(path: &Path, show_hidden: bool) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && is_hidden_name(&name) {
            continue;
        }

        let metadata = entry.metadata()?;
        let file_type = entry.file_type()?;
        let kind = if file_type.is_dir() {
            FileKind::Directory
        } else if file_type.is_symlink() {
            FileKind::Symlink
        } else {
            kind_from_name(&name)
        };

        entries.push(FileEntry {
            name: name.clone(),
            path: entry.path().display().to_string(),
            kind,
            size: metadata.len(),
            permissions: String::new(),
            modified: String::new(),
            owner: String::new(),
            group: String::new(),
            selected: false,
        });
    }

    sort_entries_for_display(&mut entries);
    Ok(entries)
}

pub fn sort_entries_for_display(entries: &mut [FileEntry]) {
    entries.sort_by(|a, b| {
        file_sort_key(a)
            .cmp(&file_sort_key(b))
            .then_with(|| a.name.cmp(&b.name))
    });
}

pub fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.') && name != "." && name != ".."
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(rest)
    } else if path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
    } else {
        path.into()
    }
}

pub fn resolve_local_path(base: &Path, input: &str) -> PathBuf {
    let expanded = expand_tilde(input);
    if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    }
}

pub fn local_file_name(path: &Path) -> Option<String> {
    path.file_name().map(|name| name.to_string_lossy().to_string())
}

fn file_sort_key(entry: &FileEntry) -> (bool, bool, String) {
    (
        entry.kind != FileKind::Directory,
        is_hidden_name(&entry.name),
        entry.name.trim_start_matches('.').to_ascii_lowercase(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, write};

    #[test]
    fn local_listing_hides_dotfiles_by_default() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path().join(".env"), "").unwrap();
        write(dir.path().join("README.md"), "").unwrap();

        let names: Vec<_> = list_dir(dir.path(), false)
            .unwrap()
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(names, vec!["README.md"]);
    }

    #[test]
    fn local_listing_sorts_directories_then_visible_then_hidden() {
        let dir = tempfile::tempdir().unwrap();
        create_dir(dir.path().join(".config")).unwrap();
        create_dir(dir.path().join("src")).unwrap();
        write(dir.path().join(".env"), "").unwrap();
        write(dir.path().join("README.md"), "").unwrap();

        let names: Vec<_> = list_dir(dir.path(), true)
            .unwrap()
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(names, vec!["src", ".config", "README.md", ".env"]);
    }

    #[test]
    fn resolves_tilde_and_relative_local_paths() {
        let base = Path::new("/tmp/base");
        assert_eq!(resolve_local_path(base, "notes.txt"), PathBuf::from("/tmp/base/notes.txt"));
        assert_eq!(expand_tilde("/var/log"), PathBuf::from("/var/log"));
    }
}
