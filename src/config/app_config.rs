use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "blackout".into()
}

fn default_downloads_dir() -> String {
    "~/Downloads".into()
}

fn default_preview_max_bytes() -> u64 {
    1_048_576
}

fn default_command() -> String {
    "ssh".into()
}

fn default_tunnel_type() -> String {
    "local".into()
}

fn default_tunnel_local_port() -> u16 {
    8080
}

fn default_tunnel_target_host() -> Option<String> {
    Some("localhost".into())
}

fn default_tunnel_target_port() -> Option<u16> {
    Some(80)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub animations: bool,
    #[serde(default = "default_true")]
    pub unicode: bool,
    #[serde(default = "default_true")]
    pub nerd_font: bool,
    #[serde(default = "default_true")]
    pub mouse: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            animations: true,
            unicode: true,
            nerd_font: true,
            mouse: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_downloads_dir")]
    pub default_local_dir: String,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_preview_max_bytes")]
    pub preview_max_bytes: u64,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            default_local_dir: default_downloads_dir(),
            show_hidden: false,
            preview_max_bytes: default_preview_max_bytes(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostMeta {
    pub tags: Vec<String>,
    pub group: Option<String>,
    pub favorite: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_command")]
    pub default_command: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_command: default_command(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastPathsConfig {
    #[serde(default)]
    pub local: Option<String>,
    #[serde(default)]
    pub remote_by_host: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelPresetConfig {
    #[serde(default = "default_tunnel_type")]
    pub tunnel_type: String,
    #[serde(default)]
    pub host_alias: String,
    #[serde(default)]
    pub bind_address: Option<String>,
    #[serde(default = "default_tunnel_local_port")]
    pub local_port: u16,
    #[serde(default = "default_tunnel_target_host")]
    pub target_host: Option<String>,
    #[serde(default = "default_tunnel_target_port")]
    pub target_port: Option<u16>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl Default for TunnelPresetConfig {
    fn default() -> Self {
        Self {
            tunnel_type: default_tunnel_type(),
            host_alias: String::new(),
            bind_address: None,
            local_port: default_tunnel_local_port(),
            target_host: default_tunnel_target_host(),
            target_port: default_tunnel_target_port(),
            notes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    pub path: PathBuf,
    #[serde(default)]
    pub hidden_imported_hosts: Vec<String>,
    #[serde(default)]
    pub recent_hosts: Vec<String>,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub hosts: BTreeMap<String, HostMeta>,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default = "default_bookmarks")]
    pub bookmarks: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub tunnel_presets: BTreeMap<String, TunnelPresetConfig>,
    #[serde(default)]
    pub last_paths: LastPathsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path: default_config_path(),
            hidden_imported_hosts: Vec::new(),
            recent_hosts: Vec::new(),
            ui: UiConfig::default(),
            hosts: BTreeMap::new(),
            files: FilesConfig::default(),
            bookmarks: default_bookmarks(),
            settings: Settings::default(),
            tunnel_presets: BTreeMap::new(),
            last_paths: LastPathsConfig::default(),
        }
    }
}

fn default_bookmarks() -> BTreeMap<String, BTreeMap<String, String>> {
    let mut bookmarks = BTreeMap::new();
    bookmarks.insert(
        "global".into(),
        BTreeMap::from([("downloads".into(), "~/Downloads".into()), ("home".into(), "~".into())]),
    );
    bookmarks
}

pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sshdeck/config.toml")
}

impl AppConfig {
    pub fn load_or_default(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = path.unwrap_or_else(default_config_path);
        if !path.exists() {
            let cfg = Self {
                path,
                ..Default::default()
            };
            cfg.save()?;
            return Ok(cfg);
        }
        let text = fs::read_to_string(&path)?;
        let mut cfg: AppConfig = toml::from_str(&text)?;
        cfg.path = path;
        Ok(cfg)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        if self.path.exists() {
            let bak = self
                .path
                .with_extension(format!("toml.bak.{}", chrono::Local::now().format("%Y%m%d-%H%M%S%.f")));
            fs::copy(&self.path, bak).ok();
        }
        let mut clone = self.clone();
        clone.path = PathBuf::new();
        atomic_write(&self.path, toml::to_string_pretty(&clone)?.as_bytes())?;
        Ok(())
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_bookmarks_from_toml() {
        let toml = r#"[ui]
theme='cyber'
animations=true
unicode=true
nerd_font=false
mouse=true
[files]
default_local_dir='~/Downloads'
show_hidden=true
preview_max_bytes=123
[settings]
default_command='ssh'
[bookmarks.global]
downloads='~/Downloads'
[bookmarks.web]
webroot='/var/www'
"#;
        let mut config: AppConfig = toml::from_str(toml).unwrap();
        config.path = PathBuf::new();
        assert_eq!(config.ui.theme, "cyber");
        assert_eq!(config.bookmarks["web"]["webroot"], "/var/www");
        assert!(config.files.show_hidden);
    }

    #[test]
    fn legacy_config_without_state_fields_uses_defaults() {
        let toml = "";
        let config: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.ui.theme, "blackout");
        assert!(config.ui.mouse);
        assert_eq!(config.files.default_local_dir, "~/Downloads");
        assert_eq!(config.settings.default_command, "ssh");
        assert!(config.hidden_imported_hosts.is_empty());
        assert!(config.recent_hosts.is_empty());
        assert!(config.tunnel_presets.is_empty());
        assert!(config.last_paths.local.is_none());
        assert_eq!(config.bookmarks["global"]["home"], "~");
    }

    #[test]
    fn loads_reserved_state_fields_from_toml() {
        let toml = r#"hidden_imported_hosts=['legacy-bastion']
recent_hosts=['web-prod-1','db-prod-1']

[ui]
theme='minimal'
animations=false
unicode=true
nerd_font=false
mouse=true

[files]
default_local_dir='~/Downloads'
show_hidden=true
preview_max_bytes=2048

[settings]
default_command='ssh'

[last_paths]
local='~/work'

[last_paths.remote_by_host]
web-prod-1='/var/www'

[tunnel_presets.web]
tunnel_type='local'
host_alias='web-prod-1'
local_port=8080
target_host='localhost'
target_port=80
notes='web app'
"#;
        let config: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.hidden_imported_hosts, vec!["legacy-bastion".to_string()]);
        assert_eq!(config.recent_hosts, vec!["web-prod-1".to_string(), "db-prod-1".to_string()]);
        assert_eq!(config.last_paths.local.as_deref(), Some("~/work"));
        assert_eq!(config.last_paths.remote_by_host["web-prod-1"], "/var/www");
        assert_eq!(config.tunnel_presets["web"].host_alias, "web-prod-1");
        assert_eq!(config.tunnel_presets["web"].target_port, Some(80));
    }
}
