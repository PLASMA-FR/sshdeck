use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<PathBuf>,
    pub proxy_jump: Option<String>,
    pub local_forwards: Vec<String>,
    pub remote_forwards: Vec<String>,
    pub forward_agent: Option<String>,
    pub server_alive_interval: Option<u64>,
    pub tags: Vec<String>,
    pub group: Option<String>,
    pub favorite: bool,
    pub notes: Option<String>,
    pub recent_connection: Option<String>,
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
}
