use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<PathBuf>,
    #[serde(default)]
    pub certificate_file: Option<PathBuf>,
    pub proxy_jump: Option<String>,
    pub local_forwards: Vec<String>,
    pub remote_forwards: Vec<String>,
    pub forward_agent: Option<String>,
    #[serde(default)]
    pub strict_host_key_checking: Option<String>,
    #[serde(default)]
    pub user_known_hosts_file: Option<PathBuf>,
    pub server_alive_interval: Option<u64>,
    pub tags: Vec<String>,
    pub group: Option<String>,
    pub favorite: bool,
    pub notes: Option<String>,
    pub recent_connection: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessProfile {
    pub auth: String,
    pub path: String,
    pub agent: String,
    pub host_key: String,
    pub forwards: String,
    pub boundary: &'static str,
    pub warnings: Vec<String>,
}

impl SshHost {
    pub fn display_user_host(&self) -> String {
        match (&self.user, &self.hostname) {
            (Some(u), Some(h)) => format!("{u}@{h}"),
            (_, Some(h)) => h.clone(),
            _ => self.alias.clone(),
        }
    }
    pub fn port_text(&self) -> String { self.port.unwrap_or(22).to_string() }
    pub fn search_blob(&self) -> String {
        format!("{} {} {} {} {} {}", self.alias, self.hostname.clone().unwrap_or_default(), self.user.clone().unwrap_or_default(), self.tags.join(" "), self.group.clone().unwrap_or_default(), self.notes.clone().unwrap_or_default())
    }

    pub fn access_profile(&self) -> AccessProfile {
        let mut warnings = Vec::new();
        if self.agent_forwarding_enabled() {
            warnings.push("Agent forwarding is on; use it only for trusted jump paths.".into());
        }
        if self.host_key_checking_disabled() {
            warnings.push("Strict host-key checking is disabled for this host.".into());
        }

        AccessProfile {
            auth: self.auth_summary(),
            path: self.access_path_summary(),
            agent: self.agent_summary(),
            host_key: self.host_key_summary(),
            forwards: self.forward_summary(),
            boundary: "SSHDeck keeps inventory; OpenSSH keeps credentials and known_hosts.",
            warnings,
        }
    }

    pub fn auth_summary(&self) -> String {
        match (&self.identity_file, &self.certificate_file) {
            (Some(identity), Some(cert)) if is_security_key_path(identity) => {
                format!("hardware-backed key + cert {}", compact_path(cert))
            }
            (Some(identity), Some(cert)) => {
                format!("key {} + cert {}", compact_path(identity), compact_path(cert))
            }
            (Some(identity), None) if is_security_key_path(identity) => {
                format!("hardware-backed key {}", compact_path(identity))
            }
            (Some(identity), None) => format!("key {}", compact_path(identity)),
            (None, Some(cert)) => format!("certificate {}", compact_path(cert)),
            (None, None) => "OpenSSH default / agent".into(),
        }
    }

    pub fn access_path_summary(&self) -> String {
        match &self.proxy_jump {
            Some(jump) if !jump.trim().is_empty() => format!("via {jump}"),
            _ => "direct".into(),
        }
    }

    pub fn agent_summary(&self) -> String {
        match self.forward_agent.as_deref().map(|value| value.trim()).filter(|value| !value.is_empty()) {
            Some(value) if is_yes(value) => "forwarding on".into(),
            Some(value) if is_no(value) => "forwarding off".into(),
            Some(value) => format!("ForwardAgent {value}"),
            None => "forwarding off".into(),
        }
    }

    pub fn host_key_summary(&self) -> String {
        let strict = self
            .strict_host_key_checking
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("OpenSSH default");
        let known_hosts = self
            .user_known_hosts_file
            .as_ref()
            .map(compact_path)
            .unwrap_or_else(|| "default known_hosts".into());
        format!("StrictHostKeyChecking {strict} · {known_hosts}")
    }

    pub fn forward_summary(&self) -> String {
        let total = self.local_forwards.len() + self.remote_forwards.len();
        match (self.local_forwards.len(), self.remote_forwards.len()) {
            (0, 0) => "no saved forwards".into(),
            (local, 0) => format!("{local} local forward{}", plural(local)),
            (0, remote) => format!("{remote} remote forward{}", plural(remote)),
            (local, remote) => format!("{total} saved forwards ({local} local, {remote} remote)"),
        }
    }

    pub fn agent_forwarding_enabled(&self) -> bool {
        self.forward_agent.as_deref().is_some_and(is_yes)
    }

    pub fn host_key_checking_disabled(&self) -> bool {
        self.strict_host_key_checking.as_deref().is_some_and(|value| {
            let value = value.trim().to_ascii_lowercase();
            matches!(value.as_str(), "no" | "off" | "false")
        })
    }
}

fn is_yes(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "yes" | "true" | "on")
}

fn is_no(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "no" | "false" | "off")
}

fn is_security_key_path(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            let name = name.to_ascii_lowercase();
            name.contains("ed25519_sk")
                || name.contains("ecdsa_sk")
                || name.ends_with("_sk")
                || name.contains("-sk")
        })
        .unwrap_or(false)
}

fn compact_path(path: &PathBuf) -> String {
    let display = path.display().to_string();
    let Some(home) = dirs::home_dir() else { return display; };
    let home = home.display().to_string();
    display
        .strip_prefix(&home)
        .map(|rest| format!("~{rest}"))
        .unwrap_or(display)
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_profile_surfaces_bastion_agent_and_host_key_risk() {
        let host = SshHost {
            alias: "prod".into(),
            identity_file: Some(PathBuf::from("~/.ssh/id_ed25519_sk")),
            proxy_jump: Some("bastion".into()),
            forward_agent: Some("yes".into()),
            strict_host_key_checking: Some("no".into()),
            local_forwards: vec!["8080 localhost:80".into()],
            ..Default::default()
        };

        let profile = host.access_profile();
        assert!(profile.auth.contains("hardware-backed key"));
        assert_eq!(profile.path, "via bastion");
        assert_eq!(profile.agent, "forwarding on");
        assert!(profile.host_key.contains("StrictHostKeyChecking no"));
        assert_eq!(profile.forwards, "1 local forward");
        assert_eq!(profile.warnings.len(), 2);
    }

    #[test]
    fn access_profile_defaults_to_openssh_boundary() {
        let host = SshHost {
            alias: "dev".into(),
            ..Default::default()
        };

        let profile = host.access_profile();
        assert_eq!(profile.auth, "OpenSSH default / agent");
        assert_eq!(profile.path, "direct");
        assert_eq!(profile.agent, "forwarding off");
        assert!(profile.host_key.contains("OpenSSH default"));
        assert_eq!(profile.boundary, "SSHDeck keeps inventory; OpenSSH keeps credentials and known_hosts.");
        assert!(profile.warnings.is_empty());
    }
}
