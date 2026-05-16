use std::process::Command;

use super::{
    file_entry::{parse_ls_line, FileEntry},
    safety::shell_quote_path,
};
use crate::{ssh::command::ssh_noninteractive_args_for, ssh::host::SshHost};

pub fn list_remote(host: &str, path: &str) -> anyhow::Result<Vec<FileEntry>> {
    list_remote_with_args(&[host.to_string()], path)
}

pub fn list_remote_host(host: &SshHost, path: &str) -> anyhow::Result<Vec<FileEntry>> {
    list_remote_with_args(&ssh_noninteractive_args_for(host), path)
}

fn list_remote_with_args(ssh_args: &[String], path: &str) -> anyhow::Result<Vec<FileEntry>> {
    let path_expr = remote_shell_path(path);
    let cmd = format!("cd -- {path_expr} && LC_ALL=C ls -la -- .");
    let out = Command::new("ssh").args(ssh_args).arg(cmd).output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        anyhow::bail!(if stderr.is_empty() { "remote ls failed".into() } else { stderr });
    }

    let base = normalize_display_path(path);
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .lines()
        .skip(1)
        .filter_map(parse_ls_line)
        .filter(|e| e.name != "." && e.name != "..")
        .map(|mut e| {
            e.path = join_remote_path(&base, &e.name);
            e
        })
        .collect())
}

pub fn preview_remote(host: &str, path: &str, max_bytes: u64) -> anyhow::Result<String> {
    let path_expr = remote_shell_path(path);
    let cmd = format!(
        "test $(wc -c < {path_expr}) -le {max_bytes} && sed -n '1,200p' {path_expr} || printf 'Preview skipped: file too large'"
    );
    let out = Command::new("ssh").arg(host).arg(cmd).output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
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
    if trimmed.is_empty() { "~".into() } else { trimmed.into() }
}

pub fn join_remote_path(base: &str, name: &str) -> String {
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
        if parent.is_empty() { "~".into() } else { format!("~/{parent}") }
    } else {
        let trimmed = path.trim_end_matches('/');
        trimmed.rsplit_once('/').map(|(p, _)| if p.is_empty() { "/".into() } else { p.into() }).unwrap_or("/".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_paths_expand_remotely() {
        assert_eq!(remote_shell_path("~"), "$HOME");
        assert_eq!(remote_shell_path("~/a b"), "$HOME/'a b'");
    }

    #[test]
    fn joins_home_paths_for_display() {
        assert_eq!(join_remote_path("~", "src"), "~/src");
        assert_eq!(join_remote_path("/var", "log"), "/var/log");
    }

    #[test]
    fn parent_paths_keep_home_anchor() {
        assert_eq!(parent_remote_path("~/src/app"), "~/src");
        assert_eq!(parent_remote_path("~/src"), "~");
        assert_eq!(parent_remote_path("/var/log"), "/var");
    }
}
