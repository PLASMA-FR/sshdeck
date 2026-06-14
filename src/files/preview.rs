use super::{
    file_entry::{FileEntry, FileKind},
    safety::{is_sensitive_path, validate_sensitive_access},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    Text(String),
    Directory(String),
    Binary(String),
    Sensitive(String),
}

pub fn preview_for(entry: &FileEntry) -> Preview {
    preview_for_with_confirmation(entry, false)
}

pub fn preview_for_with_confirmation(entry: &FileEntry, confirmed: bool) -> Preview {
    if let Err(message) = validate_sensitive_access(&entry.path, confirmed) {
        return Preview::Sensitive(message);
    }

    match entry.kind {
        FileKind::Directory => Preview::Directory(format!(
            "Directory: {}\nSize: cheap calculation skipped\nPermissions: {}",
            entry.name, entry.permissions
        )),
        FileKind::Image => Preview::Binary(format!(
            "Image metadata only: {} ({} bytes)",
            entry.name, entry.size
        )),
        FileKind::Archive => Preview::Binary(format!("Archive: {} ({} bytes)", entry.name, entry.size)),
        _ => Preview::Text(format!(
            "{}\n{} bytes\n{}",
            entry.name, entry.size, entry.permissions
        )),
    }
}

pub fn preview_requires_confirmation(entry: &FileEntry) -> bool {
    is_sensitive_path(&entry.path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, kind: FileKind) -> FileEntry {
        FileEntry {
            name: path.rsplit('/').next().unwrap_or(path).into(),
            path: path.into(),
            kind,
            size: 12,
            permissions: "-rw-------".into(),
            modified: String::new(),
            owner: String::new(),
            group: String::new(),
            selected: false,
        }
    }

    #[test]
    fn sensitive_preview_is_blocked_until_confirmed() {
        let entry = entry("~/.ssh/id_ed25519", FileKind::File);
        assert!(matches!(preview_for(&entry), Preview::Sensitive(_)));
        assert!(matches!(
            preview_for_with_confirmation(&entry, true),
            Preview::Text(_)
        ));
    }

    #[test]
    fn directories_render_metadata_preview() {
        let entry = entry("/tmp/src", FileKind::Directory);
        assert!(matches!(preview_for(&entry), Preview::Directory(text) if text.contains("Directory")));
    }
}
