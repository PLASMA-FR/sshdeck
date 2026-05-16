use std::{fs, path::PathBuf};
pub fn data_dir() -> PathBuf { dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("sshdeck") }
pub fn logs_path() -> PathBuf { data_dir().join("events.log") }
pub fn append_log(message: &str) { let path=logs_path(); if let Some(p)=path.parent(){ let _=fs::create_dir_all(p); } let line=format!("{} {}
", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message); let _=fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes())); }
pub fn read_logs() -> Vec<String> { fs::read_to_string(logs_path()).unwrap_or_default().lines().map(ToOwned::to_owned).collect() }
