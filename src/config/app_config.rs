use std::{collections::BTreeMap, fs, path::PathBuf};
use serde::{Deserialize, Serialize};

fn default_true() -> bool { true }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig { pub theme: String, pub animations: bool, pub unicode: bool, pub nerd_font: bool, #[serde(default = "default_true")] pub mouse: bool }
impl Default for UiConfig { fn default() -> Self { Self { theme: "blackout".into(), animations: true, unicode: true, nerd_font: true, mouse: true } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig { pub default_local_dir: String, pub show_hidden: bool, pub preview_max_bytes: u64 }
impl Default for FilesConfig { fn default() -> Self { Self { default_local_dir: "~/Downloads".into(), show_hidden: false, preview_max_bytes: 1_048_576 } } }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostMeta { pub tags: Vec<String>, pub group: Option<String>, pub favorite: bool, pub notes: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings { pub default_command: String }
impl Default for Settings { fn default() -> Self { Self { default_command: "ssh".into() } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip)] pub path: PathBuf,
    pub ui: UiConfig,
    #[serde(default)] pub hosts: BTreeMap<String, HostMeta>,
    pub files: FilesConfig,
    #[serde(default)] pub bookmarks: BTreeMap<String, BTreeMap<String, String>>,
    pub settings: Settings,
}
impl Default for AppConfig { fn default() -> Self { Self { path: default_config_path(), ui: UiConfig::default(), hosts: BTreeMap::new(), files: FilesConfig::default(), bookmarks: default_bookmarks(), settings: Settings::default() } } }
fn default_bookmarks() -> BTreeMap<String, BTreeMap<String, String>> { let mut b=BTreeMap::new(); b.insert("global".into(), BTreeMap::from([("downloads".into(), "~/Downloads".into()), ("home".into(), "~".into())])); b }
pub fn default_config_path() -> PathBuf { dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("sshdeck/config.toml") }
impl AppConfig {
    pub fn load_or_default(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = path.unwrap_or_else(default_config_path);
        if !path.exists() { let cfg = Self { path, ..Default::default() }; cfg.save()?; return Ok(cfg); }
        let text = fs::read_to_string(&path)?;
        let mut cfg: AppConfig = toml::from_str(&text)?;
        cfg.path = path;
        Ok(cfg)
    }
    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(parent)=self.path.parent() { fs::create_dir_all(parent)?; }
        if self.path.exists() { let bak = self.path.with_extension(format!("toml.bak.{}", chrono::Local::now().format("%Y%m%d-%H%M%S"))); fs::copy(&self.path, bak).ok(); }
        let mut clone = self.clone(); clone.path = PathBuf::new();
        fs::write(&self.path, toml::to_string_pretty(&clone)?)?; Ok(())
    }
}
#[cfg(test)]
mod tests { use super::*; #[test] fn loads_bookmarks_from_toml(){ let t=r#"[ui]
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
"#; let mut c:AppConfig=toml::from_str(t).unwrap(); c.path=PathBuf::new(); assert_eq!(c.ui.theme,"cyber"); assert_eq!(c.bookmarks["web"]["webroot"],"/var/www"); assert!(c.files.show_hidden); } }
