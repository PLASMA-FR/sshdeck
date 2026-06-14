use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    file_entry::{parse_ls_line, FileEntry},
    safety::{
        shell_quote_path, validate_delete_confirmation, validate_sensitive_access,
    },
};
use crate::{
    ssh::{
        command::{scp_download_args_for, scp_upload_args_for, ssh_noninteractive_args_for},
        host::SshHost,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteMutation {
    Mkdir { path: String },
    Touch { path: String },
    Rename { from: String, to: String },
    Chmod { mode: String, path: String },
    Chown { owner: String, path: String },
    Delete { path: String },
}

pub fn list_remote(host: &str, path: &str) -> anyhow::Result<Vec<FileEntry>> {
    list_remote_with_args(&raw_host_ssh_args(host), path)
}

pub fn list_remote_host(host: &SshHost, path: &str) -> anyhow::Result<Vec<FileEntry>> {
    list_remote_with_args(&ssh_noninteractive_args_for(host), path)
}

pub fn list_remote_with_hidden(
    host: &str,
    path: &str,
    show_hidden: bool,
) -> anyhow::Result<Vec<FileEntry>> {
    let entries = list_remote(host, path)?;
    Ok(filter_hidden_entries(entries, show_hidden))
}

pub fn list_remote_host_with_hidden(
    host: &SshHost,
    path: &str,
    show_hidden: bool,
) -> anyhow::Result<Vec<FileEntry>> {
    let entries = list_remote_host(host, path)?;
    Ok(filter_hidden_entries(entries, show_hidden))
}

fn list_remote_with_args(ssh_args: &[String], path: &str) -> anyhow::Result<Vec<FileEntry>> {
    let cmd = remote_list_command(path);
    let out = Command::new("ssh").args(ssh_args).arg(cmd).output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        anyhow::bail!(if stderr.is_empty() {
            "remote ls failed".into()
        } else {
            stderr
        });
    }

    let base = normalize_display_path(path);
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .lines()
        .skip(1)
        .filter_map(parse_ls_line)
        .filter(|entry| entry.name != "." && entry.name != "..")
        .map(|mut entry| {
            entry.path = join_remote_path(&base, &entry.name);
            entry
        })
        .collect())
}

pub fn mkdir_remote(host: &SshHost, path: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Mkdir { path: path.into() };
    run_remote_mutation_host(host, &mutation, None)?;
    Ok(format!("Created folder {path}"))
}

pub fn touch_remote(host: &SshHost, path: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Touch { path: path.into() };
    run_remote_mutation_host(host, &mutation, None)?;
    Ok(format!("Created file {path}"))
}

pub fn rename_remote(host: &SshHost, from: &str, to: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Rename {
        from: from.into(),
        to: to.into(),
    };
    run_remote_mutation_host(host, &mutation, None)?;
    Ok(format!("Renamed {from} to {to}"))
}

pub fn chmod_remote(host: &SshHost, mode: &str, path: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Chmod {
        mode: mode.into(),
        path: path.into(),
    };
    run_remote_mutation_host(host, &mutation, None)?;
    Ok(format!("Changed permissions for {path}"))
}

pub fn chown_remote(host: &SshHost, owner: &str, path: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Chown {
        owner: owner.into(),
        path: path.into(),
    };
    run_remote_mutation_host(host, &mutation, None)?;
    Ok(format!("Changed owner for {path}"))
}

pub fn delete_remote(host: &SshHost, path: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Delete { path: path.into() };
    run_remote_mutation_host(host, &mutation, None)?;
    Ok(format!("Deleted {path}"))
}

pub fn delete_remote_confirmed(host: &SshHost, path: &str, confirmation: &str) -> anyhow::Result<String> {
    let mutation = RemoteMutation::Delete { path: path.into() };
    run_remote_mutation_host(host, &mutation, Some(confirmation))?;
    Ok(format!("Deleted {path}"))
}

pub fn safe_edit_with_openssh(host: SshHost, path: String, editor: String) -> anyhow::Result<()> {
    let remote_path = require_remote_path(&path)?.to_string();
    let local_path = reserve_edit_temp_path(&remote_path)?;
    let local_display = local_path.display().to_string();

    let download = run_local_command(
        "scp",
        &scp_download_args_for(&host, &remote_path, &local_display),
        "remote edit download failed",
    );
    if let Err(error) = download {
        let _ = fs::remove_file(&local_path);
        return Err(error);
    }

    if let Err(error) = run_editor(&editor, &local_path) {
        let _ = fs::remove_file(&local_path);
        return Err(error);
    }

    let backup_path = remote_edit_backup_path(&remote_path);
    let backup_command = format!(
        "cp -- {} {}",
        remote_shell_path(&remote_path),
        remote_shell_path(&backup_path)
    );
    if let Err(error) = run_remote_shell_command(&host, &backup_command, "remote edit backup failed") {
        let _ = fs::remove_file(&local_path);
        return Err(error);
    }

    let upload = run_local_command(
        "scp",
        &scp_upload_args_for(&host, &local_display, &remote_path),
        "remote edit upload failed",
    );
    let _ = fs::remove_file(&local_path);
    upload
}

pub fn preview_remote(host: &str, path: &str, max_bytes: u64) -> anyhow::Result<String> {
    preview_remote_with_args(&raw_host_ssh_args(host), path, max_bytes, false)
}

pub fn preview_remote_confirmed(host: &str, path: &str, max_bytes: u64) -> anyhow::Result<String> {
    preview_remote_with_args(&raw_host_ssh_args(host), path, max_bytes, true)
}

pub fn preview_remote_host(
    host: &SshHost,
    path: &str,
    max_bytes: u64,
) -> anyhow::Result<String> {
    preview_remote_with_args(&ssh_noninteractive_args_for(host), path, max_bytes, false)
}

pub fn preview_remote_host_confirmed(
    host: &SshHost,
    path: &str,
    max_bytes: u64,
) -> anyhow::Result<String> {
    preview_remote_with_args(&ssh_noninteractive_args_for(host), path, max_bytes, true)
}

fn preview_remote_with_args(
    ssh_args: &[String],
    path: &str,
    max_bytes: u64,
    confirmed: bool,
) -> anyhow::Result<String> {
    validate_sensitive_access(path, confirmed).map_err(anyhow::Error::msg)?;
    let cmd = remote_preview_command(path, max_bytes);
    let out = Command::new("ssh").args(ssh_args).arg(cmd).output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        anyhow::bail!(if stderr.is_empty() {
            "remote preview failed".into()
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn run_remote_mutation(host: &str, mutation: &RemoteMutation) -> anyhow::Result<()> {
    run_remote_mutation_with_confirmation(host, mutation, None)
}

pub fn run_remote_mutation_with_confirmation(
    host: &str,
    mutation: &RemoteMutation,
    confirmation: Option<&str>,
) -> anyhow::Result<()> {
    run_remote_mutation_with_args(&raw_host_ssh_args(host), mutation, confirmation)
}

pub fn run_remote_mutation_host(
    host: &SshHost,
    mutation: &RemoteMutation,
    confirmation: Option<&str>,
) -> anyhow::Result<()> {
    run_remote_mutation_with_args(&ssh_noninteractive_args_for(host), mutation, confirmation)
}

fn run_remote_mutation_with_args(
    ssh_args: &[String],
    mutation: &RemoteMutation,
    confirmation: Option<&str>,
) -> anyhow::Result<()> {
    let cmd = remote_mutation_command_with_confirmation(mutation, confirmation)?;
    let out = Command::new("ssh").args(ssh_args).arg(cmd).output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        anyhow::bail!(if stderr.is_empty() {
            "remote mutation failed".into()
        } else {
            stderr
        });
    }
    Ok(())
}

pub fn remote_list_command(path: &str) -> String {
    let path_expr = remote_shell_path(path);
    format!("cd -- {path_expr} && LC_ALL=C ls -la -- .")
}

pub fn remote_preview_command(path: &str, max_bytes: u64) -> String {
    let path_expr = remote_shell_path(path);
    format!(
        "size=$(wc -c < {path_expr}) || exit 1; if [ \"$size\" -le {max_bytes} ]; then sed -n '1,200p' -- {path_expr}; else printf 'Preview skipped: file too large'; fi"
    )
}

pub fn remote_mutation_command(mutation: &RemoteMutation) -> anyhow::Result<String> {
    remote_mutation_command_with_confirmation(mutation, None)
}

pub fn remote_mutation_command_with_confirmation(
    mutation: &RemoteMutation,
    confirmation: Option<&str>,
) -> anyhow::Result<String> {
    match mutation {
        RemoteMutation::Mkdir { path } => {
            let path = require_remote_path(path)?;
            Ok(format!("mkdir -p -- {}", remote_shell_path(path)))
        }
        RemoteMutation::Touch { path } => {
            let path = require_remote_path(path)?;
            Ok(format!("touch -- {}", remote_shell_path(path)))
        }
        RemoteMutation::Rename { from, to } => {
            let from = require_remote_path(from)?;
            let to = require_remote_path(to)?;
            Ok(format!(
                "mv -- {} {}",
                remote_shell_path(from),
                remote_shell_path(to)
            ))
        }
        RemoteMutation::Chmod { mode, path } => {
            let path = require_remote_path(path)?;
            validate_chmod_mode(mode)?;
            Ok(format!(
                "chmod -- {} {}",
                shell_words::quote(mode),
                remote_shell_path(path)
            ))
        }
        RemoteMutation::Chown { owner, path } => {
            let path = require_remote_path(path)?;
            validate_chown_owner(owner)?;
            Ok(format!(
                "chown -- {} {}",
                shell_words::quote(owner),
                remote_shell_path(path)
            ))
        }
        RemoteMutation::Delete { path } => {
            let path = require_remote_path(path)?;
            validate_delete_confirmation(path, confirmation.unwrap_or(""))
                .map_err(anyhow::Error::msg)?;
            Ok(format!("rm -r -- {}", remote_shell_path(path)))
        }
    }
}

pub fn remote_shell_path(path: &str) -> String {
    let trimmed = path.trim();
    match trimmed {
        "" | "~" => "$HOME".into(),
        p if p.starts_with("~/") => format!("$HOME/{}", shell_quote_path(&p[2..])),
        p => shell_quote_path(p),
    }
}

pub fn normalize_display_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "~".into();
    }
    if trimmed == "~/" {
        return "~".into();
    }

    let mut normalized = trimmed.to_string();
    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
        if normalized == "~" {
            break;
        }
    }
    normalized
}

pub fn join_remote_path(base: &str, name: &str) -> String {
    let base = normalize_display_path(base);
    let name = name.trim();
    if name.is_empty() {
        return base;
    }
    if name.starts_with('/') || name == "~" || name.starts_with("~/") {
        return normalize_display_path(name);
    }

    if base == "/" {
        format!("/{name}")
    } else if base == "~" {
        format!("~/{name}")
    } else {
        format!("{}/{name}", base.trim_end_matches('/'))
    }
}

pub fn parent_remote_path(path: &str) -> String {
    let path = normalize_display_path(path);
    if path == "/" || path == "~" {
        return path;
    }
    if let Some(rest) = path.strip_prefix("~/") {
        let parent = rest.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
        if parent.is_empty() {
            "~".into()
        } else {
            format!("~/{parent}")
        }
    } else {
        let trimmed = path.trim_end_matches('/');
        trimmed
            .rsplit_once('/')
            .map(|(parent, _)| {
                if parent.is_empty() {
                    "/".into()
                } else {
                    parent.into()
                }
            })
            .unwrap_or_else(|| ".".into())
    }
}

pub fn remote_file_name(path: &str) -> Option<String> {
    let path = normalize_display_path(path);
    if path == "/" || path == "~" || path == "." {
        return None;
    }
    path.rsplit('/').next().map(str::to_string)
}

pub fn is_absolute_remote_path(path: &str) -> bool {
    let trimmed = path.trim();
    trimmed.starts_with('/') || trimmed == "~" || trimmed.starts_with("~/")
}

fn raw_host_ssh_args(host: &str) -> Vec<String> {
    vec!["--".into(), host.into()]
}

fn filter_hidden_entries(entries: Vec<FileEntry>, show_hidden: bool) -> Vec<FileEntry> {
    if show_hidden {
        entries
    } else {
        entries
            .into_iter()
            .filter(|entry| !is_hidden_remote_name(&entry.name))
            .collect()
    }
}

fn is_hidden_remote_name(name: &str) -> bool {
    name.starts_with('.') && name != "." && name != ".."
}

fn reserve_edit_temp_path(remote_path: &str) -> anyhow::Result<PathBuf> {
    let name = remote_file_name(remote_path).unwrap_or_else(|| "remote-file".into());
    let sanitized = sanitize_local_name(&name);
    let path = std::env::temp_dir().join(format!(
        "sshdeck-edit-{}-{}-{sanitized}",
        std::process::id(),
        timestamp_seconds()
    ));
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    Ok(path)
}

fn sanitize_local_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "remote-file".into()
    } else {
        sanitized
    }
}

fn remote_edit_backup_path(path: &str) -> String {
    format!(
        "{}.sshdeck.bak.{}",
        normalize_display_path(path),
        timestamp_seconds()
    )
}

fn timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn run_editor(editor: &str, path: &Path) -> anyhow::Result<()> {
    let parts = shell_words::split(editor).unwrap_or_else(|_| vec![editor.into()]);
    let (program, args) = parts
        .split_first()
        .filter(|(program, _)| !program.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("editor command cannot be empty"))?;
    let status = Command::new(program).args(args).arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("editor exited with {status}")
    }
}

fn run_remote_shell_command(host: &SshHost, command: &str, label: &str) -> anyhow::Result<()> {
    let output = Command::new("ssh")
        .args(ssh_noninteractive_args_for(host))
        .arg(command)
        .output()?;
    command_output_result(output, label)
}

fn run_local_command(program: &str, args: &[String], label: &str) -> anyhow::Result<()> {
    let output = Command::new(program).args(args).output()?;
    command_output_result(output, label)
}

fn command_output_result(output: std::process::Output, label: &str) -> anyhow::Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    anyhow::bail!(if stderr.is_empty() {
        format!("{label}: {}", output.status)
    } else {
        format!("{label}: {stderr}")
    })
}

fn require_remote_path(path: &str) -> anyhow::Result<&str> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        anyhow::bail!("remote path cannot be empty");
    }
    Ok(trimmed)
}

fn validate_chmod_mode(mode: &str) -> anyhow::Result<()> {
    let mode = mode.trim();
    let numeric = (3..=4).contains(&mode.len()) && mode.chars().all(|c| matches!(c, '0'..='7'));
    let symbolic = !mode.is_empty()
        && !mode.starts_with('-')
        && mode
            .chars()
            .all(|c| matches!(c, 'a' | 'u' | 'g' | 'o' | '+' | '-' | '=' | 'r' | 'w' | 'x' | 'X' | 's' | 't' | ',' ));
    if numeric || symbolic {
        Ok(())
    } else {
        anyhow::bail!("invalid chmod mode: {mode}");
    }
}

fn validate_chown_owner(owner: &str) -> anyhow::Result<()> {
    let owner = owner.trim();
    let valid = !owner.is_empty()
        && !owner.starts_with('-')
        && owner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'));
    if valid {
        Ok(())
    } else {
        anyhow::bail!("invalid chown owner: {owner}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_blocks_sensitive_paths_before_ssh() {
        let err = preview_remote("example.invalid", "~/.ssh/id_ed25519", 1024).unwrap_err();
        assert!(err.to_string().contains("Sensitive path requires explicit confirmation"));
    }

    #[test]
    fn raw_host_aliases_are_separated_from_ssh_options() {
        let err = list_remote("-oProxyCommand=evil", "~").unwrap_err();
        let msg = err.to_string();
        assert!(!msg.contains("ProxyCommand"));
    }

    #[test]
    fn builds_remote_preview_command_with_quoted_path() {
        let cmd = remote_preview_command("~/a b", 4096);
        assert!(cmd.contains("$HOME/'a b'"));
        assert!(cmd.contains("-le 4096"));
    }

    #[test]
    fn home_paths_expand_remotely() {
        assert_eq!(remote_shell_path("~"), "$HOME");
        assert_eq!(remote_shell_path("~/a b"), "$HOME/'a b'");
    }

    #[test]
    fn joins_home_paths_for_display() {
        assert_eq!(join_remote_path("~", "src"), "~/src");
        assert_eq!(join_remote_path("/var", "log"), "/var/log");
        assert_eq!(join_remote_path("/var/", "log"), "/var/log");
        assert_eq!(join_remote_path("~", "/tmp"), "/tmp");
    }

    #[test]
    fn parent_paths_keep_home_anchor() {
        assert_eq!(parent_remote_path("~/src/app"), "~/src");
        assert_eq!(parent_remote_path("~/src"), "~");
        assert_eq!(parent_remote_path("/var/log"), "/var");
        assert_eq!(parent_remote_path("relative/path"), "relative");
    }

    #[test]
    fn remote_path_utilities_handle_roots_and_trailing_slashes() {
        assert_eq!(normalize_display_path(" /var/log/ "), "/var/log");
        assert_eq!(normalize_display_path("~/"), "~");
        assert_eq!(remote_file_name("/var/log/nginx/"), Some("nginx".into()));
        assert_eq!(remote_file_name("/"), None);
        assert!(is_absolute_remote_path("~/src"));
        assert!(!is_absolute_remote_path("relative"));
    }

    #[test]
    fn hidden_filter_keeps_dotfiles_only_when_requested() {
        let entries = vec![entry(".env"), entry("README.md")];
        let visible: Vec<_> = filter_hidden_entries(entries.clone(), false)
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        let all: Vec<_> = filter_hidden_entries(entries, true)
            .into_iter()
            .map(|entry| entry.name)
            .collect();

        assert_eq!(visible, vec!["README.md"]);
        assert_eq!(all, vec![".env", "README.md"]);
    }

    #[test]
    fn safe_edit_helpers_create_local_and_remote_safe_names() {
        assert_eq!(sanitize_local_name("a b;rm"), "a_b_rm");
        let backup = remote_edit_backup_path("~/a b");
        assert!(backup.starts_with("~/a b.sshdeck.bak."));
    }

    #[test]
    fn mutation_commands_quote_paths_and_validate_args() {
        assert_eq!(
            remote_mutation_command(&RemoteMutation::Rename {
                from: "~/a b".into(),
                to: "/tmp/c d".into(),
            })
            .unwrap(),
            "mv -- $HOME/'a b' '/tmp/c d'"
        );
        assert!(remote_mutation_command(&RemoteMutation::Chmod {
            mode: "644;rm".into(),
            path: "/tmp/a".into(),
        })
        .is_err());
        assert!(remote_mutation_command(&RemoteMutation::Chown {
            owner: "root;rm".into(),
            path: "/tmp/a".into(),
        })
        .is_err());
    }

    #[test]
    fn destructive_mutations_require_confirmation() {
        assert!(remote_mutation_command(&RemoteMutation::Delete { path: "/etc".into() }).is_err());
        assert_eq!(
            remote_mutation_command_with_confirmation(
                &RemoteMutation::Delete { path: "/etc".into() },
                Some("DELETE /etc")
            )
            .unwrap(),
            "rm -r -- /etc"
        );
        assert!(remote_mutation_command_with_confirmation(
            &RemoteMutation::Delete { path: "/".into() },
            Some("DELETE /")
        )
        .is_err());
    }

    fn entry(name: &str) -> FileEntry {
        FileEntry {
            name: name.into(),
            path: name.into(),
            kind: super::super::file_entry::FileKind::File,
            size: 0,
            permissions: String::new(),
            modified: String::new(),
            owner: String::new(),
            group: String::new(),
            selected: false,
        }
    }
}
