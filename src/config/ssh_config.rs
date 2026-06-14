use std::{fs, path::{Path, PathBuf}};
use regex::Regex;
use crate::ssh::host::SshHost;

fn expand_tilde(s: &str) -> PathBuf { if let Some(rest) = s.strip_prefix("~/") { dirs::home_dir().unwrap_or_default().join(rest) } else { PathBuf::from(s) } }

pub fn default_ssh_config_path() -> Option<PathBuf> { dirs::home_dir().map(|h| h.join(".ssh/config")) }
pub fn parse_default_ssh_config() -> anyhow::Result<Vec<SshHost>> { match default_ssh_config_path() { Some(p) => parse_ssh_config_file(p), None => Ok(vec![]) } }
pub fn parse_ssh_config_file(path: impl AsRef<Path>) -> anyhow::Result<Vec<SshHost>> { let text = fs::read_to_string(path)?; Ok(parse_ssh_config(&text)) }

pub fn parse_ssh_config(input: &str) -> Vec<SshHost> {
    let mut hosts = Vec::new();
    let mut current: Option<SshHost> = None;
    let kv = Regex::new(r#"^\s*([^#\s]+)\s+(.+?)\s*$"#).unwrap();
    for raw in input.lines() {
        let line = raw.split('#').next().unwrap_or("").trim_end();
        if line.trim().is_empty() { continue; }
        let Some(caps) = kv.captures(line) else { continue; };
        let key = caps[1].to_ascii_lowercase();
        let val = caps[2].trim().trim_matches('"').to_string();
        if key == "host" {
            if let Some(h) = current.take() { hosts.push(h); }
            let alias = val.split_whitespace().next().unwrap_or("").to_string();
            if alias.contains('*') || alias == "*" { current = None; } else { current = Some(SshHost { alias, ..Default::default() }); }
            continue;
        }
        let Some(h) = current.as_mut() else { continue; };
        match key.as_str() {
            "hostname" => h.hostname = Some(val),
            "user" => h.user = Some(val),
            "port" => h.port = val.parse().ok(),
            "identityfile" => h.identity_file = Some(expand_tilde(&val)),
            "certificatefile" => h.certificate_file = Some(expand_tilde(&val)),
            "proxyjump" => h.proxy_jump = Some(val),
            "localforward" => h.local_forwards.push(val),
            "remoteforward" => h.remote_forwards.push(val),
            "forwardagent" => h.forward_agent = Some(val),
            "stricthostkeychecking" => h.strict_host_key_checking = Some(val),
            "userknownhostsfile" => h.user_known_hosts_file = Some(expand_tilde(&val)),
            "serveraliveinterval" => h.server_alive_interval = val.parse().ok(),
            _ => {}
        }
    }
    if let Some(h) = current { hosts.push(h); }
    hosts
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_common_ssh_config_blocks() {
        let s = r#"Host web-prod-1
  HostName 1.2.3.4
  User root
  Port 2222
  IdentityFile ~/.ssh/id_ed25519
  CertificateFile ~/.ssh/id_ed25519-cert.pub
  ProxyJump bastion
  LocalForward 8080 localhost:80
  RemoteForward 9090 localhost:90
  ForwardAgent yes
  StrictHostKeyChecking yes
  UserKnownHostsFile ~/.ssh/known_hosts_prod
  ServerAliveInterval 60
Host *
  User ignored
"#;
        let hosts = parse_ssh_config(s);
        assert_eq!(hosts.len(), 1);
        let h=&hosts[0];
        assert_eq!(h.alias, "web-prod-1"); assert_eq!(h.hostname.as_deref(), Some("1.2.3.4")); assert_eq!(h.user.as_deref(), Some("root")); assert_eq!(h.port, Some(2222)); assert_eq!(h.proxy_jump.as_deref(), Some("bastion")); assert_eq!(h.local_forwards, vec!["8080 localhost:80"]); assert_eq!(h.server_alive_interval, Some(60));
        assert!(h.certificate_file.as_ref().is_some_and(|p| p.ends_with(".ssh/id_ed25519-cert.pub")));
        assert_eq!(h.strict_host_key_checking.as_deref(), Some("yes"));
        assert!(h.user_known_hosts_file.as_ref().is_some_and(|p| p.ends_with(".ssh/known_hosts_prod")));
    }
}
