use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationRequirement {
    NotRequired,
    Required { phrase: String, reason: String },
    Blocked { reason: String },
}

impl ConfirmationRequirement {
    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required { .. })
    }

    pub fn phrase(&self) -> Option<&str> {
        match self {
            Self::Required { phrase, .. } => Some(phrase),
            _ => None,
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Required { reason, .. } | Self::Blocked { reason } => Some(reason),
            Self::NotRequired => None,
        }
    }
}

pub fn shell_quote_path(path: &str) -> String {
    shell_words::quote(path).to_string()
}

pub fn is_sensitive_path(path: &str) -> bool {
    let normalized = normalized_confirmation_path(path).to_ascii_lowercase();
    let name = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    name == ".env"
        || name.starts_with(".env.")
        || name == "authorized_keys"
        || name == "id_rsa"
        || name == "id_ed25519"
        || name == "id_ecdsa"
        || name == "id_dsa"
        || name == "id_rsa.pub"
        || name == "id_ed25519.pub"
        || normalized == "/etc/shadow"
        || normalized == "/etc/passwd"
        || normalized.contains("private_key")
        || normalized.ends_with(".pem")
        || normalized.ends_with(".key")
}

pub fn sensitive_confirmation_requirement(path: &str) -> ConfirmationRequirement {
    if is_sensitive_path(path) {
        ConfirmationRequirement::Required {
            phrase: sensitive_confirmation_phrase(path),
            reason: format!(
                "Sensitive path requires typed confirmation: {}",
                normalized_confirmation_path(path)
            ),
        }
    } else {
        ConfirmationRequirement::NotRequired
    }
}

pub fn sensitive_confirmation_phrase(path: &str) -> String {
    format!("PREVIEW {}", normalized_confirmation_path(path))
}

pub fn validate_sensitive_access(path: &str, confirmed: bool) -> Result<(), String> {
    if !confirmed && is_sensitive_path(path) {
        return Err(format!(
            "Sensitive path requires explicit confirmation: {}",
            normalized_confirmation_path(path)
        ));
    }
    Ok(())
}

pub fn validate_sensitive_confirmation(path: &str, typed: &str) -> Result<(), String> {
    match sensitive_confirmation_requirement(path) {
        ConfirmationRequirement::NotRequired => Ok(()),
        ConfirmationRequirement::Blocked { reason } => Err(reason),
        ConfirmationRequirement::Required { phrase, reason } => {
            if typed_confirmation_matches(&phrase, typed) {
                Ok(())
            } else {
                Err(format!("{reason}. Type '{phrase}' to continue."))
            }
        }
    }
}

pub fn is_dangerous_delete_path(path: &str) -> bool {
    matches!(
        normalized_confirmation_path(path).as_str(),
        "/" | "~" | "/etc" | "/usr" | "/bin" | "/sbin" | "/home" | "/root" | "/var" | "/opt"
    )
}

pub fn destructive_delete_confirmation_requirement(path: &str) -> ConfirmationRequirement {
    let normalized = normalized_confirmation_path(path);
    if normalized == "/" {
        return ConfirmationRequirement::Blocked {
            reason: "SSHDeck never allows deleting /".into(),
        };
    }

    if is_dangerous_delete_path(&normalized) {
        ConfirmationRequirement::Required {
            phrase: destructive_delete_confirmation_phrase(&normalized),
            reason: format!("Dangerous path requires typed confirmation: {normalized}"),
        }
    } else {
        ConfirmationRequirement::NotRequired
    }
}

pub fn destructive_delete_confirmation_phrase(path: &str) -> String {
    format!("DELETE {}", normalized_confirmation_path(path))
}

pub fn validate_delete(path: &str) -> Result<(), String> {
    match destructive_delete_confirmation_requirement(path) {
        ConfirmationRequirement::NotRequired => Ok(()),
        ConfirmationRequirement::Required { reason, .. } | ConfirmationRequirement::Blocked { reason } => {
            Err(reason)
        }
    }
}

pub fn validate_delete_confirmation(path: &str, typed: &str) -> Result<(), String> {
    match destructive_delete_confirmation_requirement(path) {
        ConfirmationRequirement::NotRequired => Ok(()),
        ConfirmationRequirement::Blocked { reason } => Err(reason),
        ConfirmationRequirement::Required { phrase, reason } => {
            if typed_confirmation_matches(&phrase, typed) {
                Ok(())
            } else {
                Err(format!("{reason}. Type '{phrase}' to continue."))
            }
        }
    }
}

pub fn typed_confirmation_matches(expected: &str, typed: &str) -> bool {
    expected == typed.trim()
}

pub fn normalized_confirmation_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return ".".into();
    }
    if trimmed == "~/" {
        return "~".into();
    }

    let mut normalized = trimmed.to_string();
    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
        if normalized == "~" {
            break;
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_sensitive_paths() {
        assert!(is_sensitive_path("/var/www/.env"));
        assert!(is_sensitive_path("/var/www/.env.local"));
        assert!(is_sensitive_path("~/.ssh/id_ed25519"));
        assert!(!is_sensitive_path("README.md"));
    }

    #[test]
    fn refuses_root_delete() {
        assert!(validate_delete("/").is_err());
        assert!(validate_delete("/tmp/file").is_ok());
    }

    #[test]
    fn dangerous_delete_requires_typed_phrase() {
        let requirement = destructive_delete_confirmation_requirement("/etc/");
        assert_eq!(requirement.phrase(), Some("DELETE /etc"));
        assert!(validate_delete_confirmation("/etc", "DELETE /etc").is_ok());
        assert!(validate_delete_confirmation("/etc", "delete /etc").is_err());
    }

    #[test]
    fn sensitive_confirmation_uses_preview_phrase() {
        let requirement = sensitive_confirmation_requirement(" ~/.ssh/id_rsa ");
        assert_eq!(requirement.phrase(), Some("PREVIEW ~/.ssh/id_rsa"));
        assert!(validate_sensitive_access("~/.ssh/id_rsa", false).is_err());
        assert!(validate_sensitive_confirmation("~/.ssh/id_rsa", "PREVIEW ~/.ssh/id_rsa").is_ok());
    }

    #[test]
    fn quotes_remote_paths() {
        assert_eq!(shell_quote_path("/tmp/a b"), "'/tmp/a b'");
    }

    #[test]
    fn normalizes_confirmation_paths() {
        assert_eq!(normalized_confirmation_path(" /var/log/ "), "/var/log");
        assert_eq!(normalized_confirmation_path("~/"), "~");
        assert_eq!(normalized_confirmation_path(""), ".");
    }
}
