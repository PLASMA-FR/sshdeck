use super::{file_entry::{FileEntry, FileKind}, safety::is_sensitive_path};
pub enum Preview { Text(String), Directory(String), Binary(String), Sensitive(String) }
pub fn preview_for(entry:&FileEntry)->Preview{ if is_sensitive_path(&entry.path){ return Preview::Sensitive("Sensitive file preview blocked until confirmation".into()); } match entry.kind { FileKind::Directory=>Preview::Directory(format!("Directory: {}
Size: cheap calculation skipped
Permissions: {}", entry.name, entry.permissions)), FileKind::Image=>Preview::Binary(format!("Image metadata only: {} ({} bytes)", entry.name, entry.size)), FileKind::Archive=>Preview::Binary(format!("Archive: {} ({} bytes)", entry.name, entry.size)), _=>Preview::Text(format!("{}
{} bytes
{}", entry.name, entry.size, entry.permissions)) } }
