use std::collections::BTreeMap; use crate::config::app_config::AppConfig;
pub fn default_remote_places()->Vec<&'static str>{ vec!["~","/","/var/www","/etc","/tmp","/opt","/home"] }
pub fn default_local_places()->Vec<&'static str>{ vec!["~","~/Downloads","~/Documents","."] }
pub fn bookmarks_for(cfg:&AppConfig, host:&str)->BTreeMap<String,String>{ let mut m=cfg.bookmarks.get("global").cloned().unwrap_or_default(); if let Some(h)=cfg.bookmarks.get(host){ for (k,v) in h { m.insert(k.clone(), v.clone()); }} m }
