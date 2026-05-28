use std::{fs, io::{self, Write}, path::{Path, PathBuf}};

use chrono::Local;

use crate::ssh::host::SshHost;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostDraft {
    pub alias: String,
    pub hostname: String,
    pub user: String,
    pub port: String,
    pub identity_file: String,
    pub group: String,
    pub tags: String,
    pub notes: String,
}

impl Default for HostDraft {
    fn default() -> Self {
        Self { alias: String::new(), hostname: String::new(), user: std::env::var("USER").unwrap_or_else(|_| "root".into()), port: "22".into(), identity_file: default_identity_file(), group: String::new(), tags: String::new(), notes: String::new() }
    }
}

impl HostDraft {
    pub fn from_host(host: &SshHost) -> Self {
        Self { alias: host.alias.clone(), hostname: host.hostname.clone().unwrap_or_default(), user: host.user.clone().unwrap_or_else(|| std::env::var("USER").unwrap_or_default()), port: host.port.unwrap_or(22).to_string(), identity_file: host.identity_file.as_ref().map(|p| p.display().to_string()).unwrap_or_default(), group: host.group.clone().unwrap_or_default(), tags: host.tags.join(","), notes: host.notes.clone().unwrap_or_default() }
    }
    pub fn to_host(&self) -> Option<SshHost> {
        Some(SshHost { alias: self.alias.trim().to_string(), hostname: Some(self.hostname.trim().to_string()), user: empty_none(&self.user), port: Some(self.port.trim().parse().ok()?), identity_file: empty_none(&self.identity_file).map(PathBuf::from), tags: self.tags.split(',').map(str::trim).filter(|s| !s.is_empty()).map(ToOwned::to_owned).collect(), group: empty_none(&self.group), notes: empty_none(&self.notes), ..Default::default() })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostValidationLevel { Error, Warning }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostValidationMessage { pub level: HostValidationLevel, pub message: String }

pub fn managed_config_path() -> PathBuf { dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("sshdeck/ssh_config") }
pub fn backup_dir() -> PathBuf { dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("sshdeck/backups") }

pub fn validate_host_draft(draft: &HostDraft, existing_aliases: &[String]) -> Vec<HostValidationMessage> {
    let mut messages = Vec::new();
    if draft.alias.trim().is_empty() { err(&mut messages, "Alias is required"); }
    if draft.hostname.trim().is_empty() { err(&mut messages, "Hostname/IP is required"); }
    if draft.port.trim().is_empty() { err(&mut messages, "Port is required"); }
    else if draft.port.trim().parse::<u16>().is_err() { err(&mut messages, "Port must be numeric and between 1 and 65535"); }
    for (label, value) in [
        ("Alias", draft.alias.as_str()),
        ("Hostname/IP", draft.hostname.as_str()),
        ("User", draft.user.as_str()),
        ("Identity file", draft.identity_file.as_str()),
        ("Group", draft.group.as_str()),
        ("Tags", draft.tags.as_str()),
        ("Notes", draft.notes.as_str()),
    ] {
        if contains_config_control(value) {
            err(&mut messages, format!("{label} cannot contain newlines or control characters"));
        }
    }
    if draft.alias.trim().starts_with('-') { err(&mut messages, "Alias cannot start with '-' because OpenSSH would treat it as an option"); }
    if draft.alias.contains(char::is_whitespace) { warn(&mut messages, "Alias contains spaces; OpenSSH aliases work best without spaces"); }
    if existing_aliases.iter().any(|a| a == draft.alias.trim()) { warn(&mut messages, format!("Alias '{}' already exists; save will require overwrite/rename confirmation", draft.alias.trim())); }
    if !draft.identity_file.trim().is_empty() {
        let p = expand_tilde(draft.identity_file.trim());
        if !p.exists() { warn(&mut messages, format!("Identity file does not exist: {}", draft.identity_file.trim())); }
    }
    messages
}

pub fn render_managed_host(host: &SshHost) -> String {
    let mut out = String::new();
    out.push_str(&format!("Host {}\n", one_line(&host.alias)));
    if let Some(v) = &host.hostname { out.push_str(&format!("  HostName {}\n", one_line(v))); }
    if let Some(v) = &host.user { out.push_str(&format!("  User {}\n", one_line(v))); }
    out.push_str(&format!("  Port {}\n", host.port.unwrap_or(22)));
    if let Some(v) = &host.identity_file { out.push_str(&format!("  IdentityFile {}\n", one_line(&v.display().to_string()))); }
    if let Some(v) = &host.proxy_jump { out.push_str(&format!("  ProxyJump {}\n", one_line(v))); }
    for v in &host.local_forwards { out.push_str(&format!("  LocalForward {}\n", one_line(v))); }
    for v in &host.remote_forwards { out.push_str(&format!("  RemoteForward {}\n", one_line(v))); }
    if let Some(v) = &host.forward_agent { out.push_str(&format!("  ForwardAgent {}\n", one_line(v))); }
    if let Some(v) = host.server_alive_interval { out.push_str(&format!("  ServerAliveInterval {}\n", v)); }
    out.push('\n');
    out
}

pub fn save_managed_hosts(path: &Path, hosts: &[SshHost]) -> io::Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let backups = path.parent().unwrap_or_else(|| Path::new(".")).join("backups");
    let _ = backup_file(path, &backups)?;
    let content: String = hosts.iter().map(render_managed_host).collect();
    atomic_write(path, content.as_bytes())
}

pub fn read_managed_hosts(path: &Path) -> io::Result<Vec<SshHost>> {
    if !path.exists() { return Ok(Vec::new()); }
    Ok(crate::config::ssh_config::parse_ssh_config(&fs::read_to_string(path)?))
}

pub fn backup_file(path: &Path, backup_dir: &Path) -> io::Result<Option<PathBuf>> {
    if !path.exists() { return Ok(None); }
    fs::create_dir_all(backup_dir)?;
    let stamp = Local::now().format("%Y%m%d-%H%M%S%.f");
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("config");
    let backup = backup_dir.join(format!("{}.bak.{}", name, stamp));
    fs::copy(path, &backup)?;
    Ok(Some(backup))
}

pub fn ensure_include_line(user_config: &Path, include_line: &str) -> io::Result<bool> {
    if let Some(parent) = user_config.parent() { fs::create_dir_all(parent)?; }
    let existing = fs::read_to_string(user_config).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == include_line.trim()) { return Ok(false); }
    let backups = backup_dir();
    let _ = backup_file(user_config, &backups)?;
    let mut new_content = existing;
    if !new_content.ends_with('\n') && !new_content.is_empty() { new_content.push('\n'); }
    new_content.push_str(include_line);
    new_content.push('\n');
    atomic_write(user_config, new_content.as_bytes())?;
    Ok(true)
}

fn contains_config_control(value: &str) -> bool {
    value.chars().any(|c| c == '\n' || c == '\r' || c == '\0' || (c.is_control() && c != '\t'))
}
fn one_line(value: &str) -> String {
    value.chars().map(|c| if c == '\n' || c == '\r' || c == '\0' || (c.is_control() && c != '\t') { ' ' } else { c }).collect()
}
fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(tmp, path)
}

fn err(messages: &mut Vec<HostValidationMessage>, message: impl Into<String>) { messages.push(HostValidationMessage { level: HostValidationLevel::Error, message: message.into() }); }
fn warn(messages: &mut Vec<HostValidationMessage>, message: impl Into<String>) { messages.push(HostValidationMessage { level: HostValidationLevel::Warning, message: message.into() }); }
fn empty_none(s: &str) -> Option<String> { let t=s.trim(); if t.is_empty(){None}else{Some(t.to_string())} }
fn expand_tilde(p: &str) -> PathBuf { if let Some(rest)=p.strip_prefix("~/") { dirs::home_dir().unwrap_or_default().join(rest) } else { PathBuf::from(p) } }
fn default_identity_file() -> String { for n in ["id_ed25519","id_rsa"] { let p=dirs::home_dir().unwrap_or_default().join(".ssh").join(n); if p.exists(){ return format!("~/.ssh/{n}"); } } String::new() }

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn validates_required_alias_hostname_numeric_port_and_duplicate_alias() {
        let draft = HostDraft { alias: "web prod".into(), hostname: "".into(), user: "root".into(), port: "abc".into(), identity_file: "".into(), group: "Production".into(), tags: "web,docker".into(), notes: "".into() };
        let messages = validate_host_draft(&draft, &["web prod".into()]);
        assert!(messages.iter().any(|m| m.level == HostValidationLevel::Error && m.message.contains("Hostname")));
        assert!(messages.iter().any(|m| m.level == HostValidationLevel::Error && m.message.contains("Port")));
        assert!(messages.iter().any(|m| m.level == HostValidationLevel::Warning && m.message.contains("spaces")));
        assert!(messages.iter().any(|m| m.level == HostValidationLevel::Warning && m.message.contains("already exists")));
    }

    #[test]
    fn rejects_config_injection_control_characters() {
        let draft = HostDraft {
            alias: "web".into(),
            hostname: "example.com\n  ProxyCommand sh -c evil".into(),
            user: "root".into(),
            port: "22".into(),
            identity_file: "".into(),
            group: "".into(),
            tags: "".into(),
            notes: "".into(),
        };
        let messages = validate_host_draft(&draft, &[]);
        assert!(messages.iter().any(|m| m.level == HostValidationLevel::Error && m.message.contains("control")));
    }

    #[test]
    fn rejects_aliases_that_look_like_ssh_options() {
        let draft = HostDraft { alias: "-oProxyCommand=evil".into(), hostname: "example.com".into(), user: "root".into(), port: "22".into(), identity_file: "".into(), group: "".into(), tags: "".into(), notes: "".into() };
        let messages = validate_host_draft(&draft, &[]);
        assert!(messages.iter().any(|m| m.level == HostValidationLevel::Error && m.message.contains("cannot start")));
    }

    #[test]
    fn renders_managed_openssh_host_without_metadata() {
        let host = SshHost { alias: "my-vps".into(), hostname: Some("192.168.1.20".into()), user: Some("root".into()), port: Some(2222), identity_file: Some(PathBuf::from("~/.ssh/id_ed25519")), proxy_jump: Some("bastion".into()), tags: vec!["production".into()], ..Default::default() };
        let rendered = render_managed_host(&host);
        assert!(rendered.contains("Host my-vps\n"));
        assert!(rendered.contains("  HostName 192.168.1.20\n"));
        assert!(rendered.contains("  User root\n"));
        assert!(rendered.contains("  Port 2222\n"));
        assert!(rendered.contains("  IdentityFile ~/.ssh/id_ed25519\n"));
        assert!(rendered.contains("  ProxyJump bastion\n"));
        assert!(!rendered.contains("production"));
    }

    #[test]
    fn backup_file_creates_timestamped_copy() {
        let base = std::env::temp_dir().join(format!("sshdeck-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let src = base.join("ssh_config");
        let backups = base.join("backups");
        fs::write(&src, "Host old\n").unwrap();
        let backup = backup_file(&src, &backups).unwrap().expect("backup path");
        assert!(backup.exists());
        assert_eq!(fs::read_to_string(backup).unwrap(), "Host old\n");
        let _ = fs::remove_dir_all(&base);
    }
}
