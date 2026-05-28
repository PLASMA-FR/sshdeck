use std::{fs, io::Write, path::PathBuf};

pub fn data_dir() -> PathBuf { dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("sshdeck") }
pub fn logs_path() -> PathBuf { data_dir().join("events.log") }

pub fn append_log(message: &str) {
    let path = logs_path();
    if let Some(p) = path.parent() { let _ = fs::create_dir_all(p); }
    let line = format!("{} {}
", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), redact_log_message(message));
    let _ = fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| f.write_all(line.as_bytes()));
}

pub fn read_logs() -> Vec<String> {
    fs::read_to_string(logs_path()).unwrap_or_default().lines().rev().take(500).map(ToOwned::to_owned).collect::<Vec<_>>().into_iter().rev().collect()
}

pub fn redact_log_message(message: &str) -> String {
    let mut out = message.to_string();
    for marker in [".env", "id_rsa", "id_ed25519", "authorized_keys", "/etc/shadow"] {
        if out.contains(marker) {
            out = out.replace(marker, "[REDACTED-PATH]");
        }
    }
    let parts: Vec<&str> = out.split_whitespace().collect();
    let mut redacted = Vec::with_capacity(parts.len());
    let mut skip_next = false;
    for part in parts {
        if skip_next {
            redacted.push("[REDACTED-IDENTITY]");
            skip_next = false;
            continue;
        }
        if part == "-i" || part == "IdentityFile" {
            redacted.push(part);
            skip_next = true;
        } else {
            redacted.push(part);
        }
    }
    redacted.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_sensitive_paths_and_identity_files() {
        let msg = redact_log_message("Connecting: ssh -i ~/.ssh/id_ed25519 root@host and /srv/.env");
        assert!(!msg.contains("id_ed25519"));
        assert!(!msg.contains(".env"));
        assert!(msg.contains("[REDACTED"));
    }
}
