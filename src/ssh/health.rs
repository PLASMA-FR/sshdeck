use std::{path::Path, process::Command, time::Duration};

use crate::{
    config::{
        app_config::AppConfig,
        ssh_config::{default_ssh_config_path, parse_default_ssh_config},
    },
    ssh::{command::run_ssh_command_for, host::SshHost},
};
#[derive(Debug, Clone, Default)]
pub struct HealthInfo {
    pub uptime: String,
    pub kernel: String,
    pub memory: String,
    pub disk: String,
    pub failed_services: u32,
    pub docker_containers: u32,
}

impl HealthInfo {
    pub fn empty() -> Self {
        Self {
            uptime: "unknown".into(),
            kernel: "unknown".into(),
            memory: "unknown".into(),
            disk: "unknown".into(),
            failed_services: 0,
            docker_containers: 0,
        }
    }
}

pub struct DoctorReport {
    checks: Vec<(String, bool, String)>,
}

fn exists(bin: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

impl DoctorReport {
    pub fn run(cfg: &AppConfig) -> Self {
        let mut checks = Vec::new();
        checks.push(("ssh binary".into(), exists("ssh"), "required for connections".into()));
        checks.push(("scp binary".into(), exists("scp"), "required for transfers".into()));
        checks.push(("sftp binary".into(), exists("sftp"), "required for batch SFTP".into()));
        checks.push(("ssh-keygen binary".into(), exists("ssh-keygen"), "needed for key and certificate workflows".into()));
        checks.push(("ssh-add binary".into(), exists("ssh-add"), "needed for agent key management".into()));
        checks.push(("ssh-agent binary".into(), exists("ssh-agent"), "needed when you want agent-backed keys".into()));
        checks.push(("security-key auth".into(), true, security_key_support_detail()));
        checks.push((
            "agent socket".into(),
            true,
            std::env::var("SSH_AUTH_SOCK")
                .map(|sock| format!("SSH_AUTH_SOCK={sock}"))
                .unwrap_or_else(|_| "not set; OpenSSH can still prompt or use key files".into()),
        ));

        let term = std::env::var("TERM").unwrap_or_default();
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();
        checks.push(("terminal mouse reporting".into(), !term.is_empty() && term != "dumb", format!("TERM={term}")));
        checks.push(("mouse enabled".into(), cfg.ui.mouse, "[ui].mouse".into()));
        checks.push((
            "color support".into(),
            term.contains("color") || colorterm.contains("truecolor") || colorterm.contains("24bit"),
            format!("TERM={term} COLORTERM={colorterm}"),
        ));
        checks.push(("unicode mode".into(), cfg.ui.unicode, format!("unicode={} ascii fallback available", cfg.ui.unicode)));
        checks.push(("nerd font mode".into(), cfg.ui.nerd_font, format!("nerd_font={}", cfg.ui.nerd_font)));

        let size = std::env::var("COLUMNS")
            .ok()
            .zip(std::env::var("LINES").ok())
            .map(|(cols, lines)| format!("{cols}x{lines}"))
            .unwrap_or_else(|| "unknown".into());
        checks.push(("terminal size".into(), true, size));

        let ssh_cfg = default_ssh_config_path();
        checks.push(("~/.ssh/config".into(), ssh_cfg.as_ref().is_some_and(|p| p.exists()), "missing is OK; add hosts in SSHDeck".into()));
        let parsed = parse_default_ssh_config();
        checks.push((
            "SSH config parse".into(),
            parsed.is_ok(),
            parsed.as_ref().map(|h| format!("{} host(s)", h.len())).unwrap_or_else(|e| e.to_string()),
        ));
        checks.push((
            "managed SSH config".into(),
            crate::config::managed_hosts::managed_config_path().parent().is_some_and(Path::exists),
            crate::config::managed_hosts::managed_config_path().display().to_string(),
        ));

        if let Some(home) = dirs::home_dir() {
            let ssh = home.join(".ssh");
            let (ok, detail) = ssh_dir_status(&ssh);
            checks.push(("~/.ssh permissions".into(), ok, detail));
            let known_hosts = ssh.join("known_hosts");
            checks.push((
                "known_hosts".into(),
                known_hosts.exists(),
                if known_hosts.exists() {
                    known_hosts.display().to_string()
                } else {
                    "missing; OpenSSH will ask before trusting first host key".into()
                },
            ));
        }

        for host in parsed.unwrap_or_default() {
            if let Some(key) = host.identity_file {
                checks.push((format!("key for {}", host.alias), key.exists(), key.display().to_string()));
            }
            if let Some(cert) = host.certificate_file {
                checks.push((format!("cert for {}", host.alias), cert.exists(), cert.display().to_string()));
            }
            if let Some(known_hosts) = host.user_known_hosts_file {
                checks.push((format!("known_hosts for {}", host.alias), known_hosts.exists(), known_hosts.display().to_string()));
            }
        }

        checks.push(("app config validity".into(), true, cfg.path.display().to_string()));
        checks.push(("default local files dir".into(), expand_tilde(&cfg.files.default_local_dir).exists(), cfg.files.default_local_dir.clone()));

        Self { checks }
    }

    pub fn render_text(&self) -> String {
        let mut text = String::from("SSHDeck doctor\n\n");
        for (name, ok, detail) in &self.checks {
            text.push_str(&format!("{} {:28} {}\n", if *ok { "✓" } else { "⚠" }, name, detail));
        }
        text
    }
}

fn expand_tilde(p: &str) -> std::path::PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        dirs::home_dir().unwrap_or_default().join(rest)
    } else {
        p.into()
    }
}

fn security_key_support_detail() -> String {
    let output = Command::new("ssh").args(["-Q", "key"]).output();
    match output {
        Ok(output) if output.status.success() && supports_security_key_auth(&String::from_utf8_lossy(&output.stdout)) => {
            "local OpenSSH advertises FIDO/security-key key types".into()
        }
        Ok(output) if output.status.success() => {
            "local OpenSSH does not advertise FIDO/security-key key types".into()
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.is_empty() { "ssh -Q key failed".into() } else { stderr }
        }
        Err(err) => format!("could not run ssh -Q key: {err}"),
    }
}

fn supports_security_key_auth(keys: &str) -> bool {
    let lower = keys.to_ascii_lowercase();
    lower.contains("sk-ssh") || lower.contains("ed25519-sk") || lower.contains("ecdsa-sk")
}

fn ssh_dir_status(path: &Path) -> (bool, String) {
    if !path.exists() {
        return (false, path.display().to_string());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let mode = metadata.permissions().mode() & 0o777;
            return (
                mode & 0o077 == 0,
                format!("{} mode {:03o}", path.display(), mode),
            );
        }
    }

    (true, path.display().to_string())
}
pub fn health_commands() -> [&'static str; 6] {
    [
        "uptime 2>&1 || true",
        "df -h / 2>&1 || df -h 2>&1 || true",
        "free -h 2>&1 || true",
        "uname -a 2>&1 || true",
        "if command -v systemctl >/dev/null 2>&1; then systemctl --failed --no-pager --plain --no-legend 2>&1 || true; else printf 'systemctl not installed\\n'; fi",
        "if command -v docker >/dev/null 2>&1; then docker ps --format '{{.Names}}' 2>&1 || true; else printf 'docker not installed\\n'; fi",
    ]
}

pub fn remote_health_script() -> String {
    let commands = health_commands();
    [
        "printf '__SSHDECK_UPTIME__\\n'",
        commands[0],
        "printf '\\n__SSHDECK_DF__\\n'",
        commands[1],
        "printf '\\n__SSHDECK_FREE__\\n'",
        commands[2],
        "printf '\\n__SSHDECK_UNAME__\\n'",
        commands[3],
        "printf '\\n__SSHDECK_SYSTEMD__\\n'",
        commands[4],
        "printf '\\n__SSHDECK_DOCKER__\\n'",
        commands[5],
    ].join("; ")
}

pub fn run_remote_health(host: &SshHost, timeout: Duration) -> anyhow::Result<HealthInfo> {
    let output = run_ssh_command_for(host, &remote_health_script(), timeout, 24 * 1024)?;
    Ok(summarize(host, &output))
}

pub fn summarize(_host: &SshHost, output: &str) -> HealthInfo {
    if !output.contains("__SSHDECK_") {
        return summarize_unmarked(output);
    }

    let uptime = section(output, "__SSHDECK_UPTIME__")
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("unknown")
        .trim()
        .to_string();
    let kernel = section(output, "__SSHDECK_UNAME__")
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("unknown")
        .trim()
        .to_string();
    let memory = parse_memory(section(output, "__SSHDECK_FREE__"));
    let disk = parse_disk(section(output, "__SSHDECK_DF__"));
    let systemd = section(output, "__SSHDECK_SYSTEMD__");
    let docker = section(output, "__SSHDECK_DOCKER__");
    let failed_services = parse_failed_services(systemd);
    let docker_containers = parse_docker_containers(docker);

    HealthInfo { uptime, kernel, memory, disk, failed_services, docker_containers }
}

fn summarize_unmarked(output: &str) -> HealthInfo {
    HealthInfo {
        uptime: output.lines().find(|l| !l.trim().is_empty()).unwrap_or("unknown").into(),
        kernel: output.lines().find(|l| l.contains("Linux")).unwrap_or("unknown").into(),
        memory: "see free -h".into(),
        disk: "see df -h".into(),
        failed_services: parse_failed_services(output),
        docker_containers: 0,
    }
}

fn parse_memory(output: &str) -> String {
    output
        .lines()
        .find(|l| l.trim_start().starts_with("Mem:"))
        .unwrap_or("see free -h")
        .trim()
        .to_string()
}

fn parse_disk(output: &str) -> String {
    output
        .lines()
        .find(|l| l.split_whitespace().last() == Some("/") || l.trim_start().starts_with('/'))
        .or_else(|| {
            output
                .lines()
                .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with("Filesystem"))
        })
        .unwrap_or("see df -h")
        .trim()
        .to_string()
}

fn parse_failed_services(output: &str) -> u32 {
    if is_optional_tool_unavailable(output, "systemctl") {
        return 0;
    }

    output
        .lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains(".service") && lower.contains("failed")
        })
        .count() as u32
}

fn parse_docker_containers(output: &str) -> u32 {
    if is_optional_tool_unavailable(output, "docker") {
        return 0;
    }

    output
        .lines()
        .filter(|line| {
            let lower = line.trim().to_ascii_lowercase();
            !lower.is_empty()
                && !lower.contains("permission denied")
                && !lower.contains("cannot connect to the docker daemon")
                && !lower.starts_with("error")
        })
        .count() as u32
}

fn is_optional_tool_unavailable(output: &str, tool: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    lower.contains(&format!("{tool} not installed"))
        || lower.contains(&format!("{tool}: not found"))
        || lower.contains(&format!("command not found: {tool}"))
}

fn section<'a>(output: &'a str, marker: &str) -> &'a str {
    let Some((_, rest)) = output.split_once(marker) else { return ""; };
    rest.split("__SSHDECK_").next().unwrap_or(rest).trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_marked_remote_health_output() {
        let output = "__SSHDECK_UPTIME__
 10:23:10 up 2 days
__SSHDECK_DF__
Filesystem Size Used Avail Use% Mounted on
/dev/sda1 50G 20G 30G 40% /
__SSHDECK_FREE__
Mem: 7.7Gi 2.0Gi 5.0Gi
__SSHDECK_UNAME__
Linux demo 6.1
__SSHDECK_SYSTEMD__
demo.service loaded failed failed demo
__SSHDECK_DOCKER__
web
db
";
        let info = summarize(&SshHost::default(), output);
        assert!(info.uptime.contains("up 2 days"));
        assert_eq!(info.failed_services, 1);
        assert_eq!(info.docker_containers, 2);
    }

    #[test]
    fn remote_health_script_guards_optional_tools() {
        let script = remote_health_script();
        assert!(script.contains("command -v systemctl"));
        assert!(script.contains("systemctl not installed"));
        assert!(script.contains("command -v docker"));
        assert!(script.contains("docker not installed"));
    }

    #[test]
    fn missing_optional_tools_count_as_zero() {
        let output = "__SSHDECK_UPTIME__
 up 4 hours
__SSHDECK_DF__
Filesystem Size Used Avail Use% Mounted on
/dev/sda1 50G 20G 30G 40% /
__SSHDECK_FREE__
Mem: 7.7Gi 2.0Gi 5.0Gi
__SSHDECK_UNAME__
Linux demo 6.1
__SSHDECK_SYSTEMD__
systemctl not installed
__SSHDECK_DOCKER__
docker not installed
";
        let info = summarize(&SshHost::default(), output);
        assert_eq!(info.failed_services, 0);
        assert_eq!(info.docker_containers, 0);
    }

    #[test]
    fn detects_security_key_key_types_from_openssh_query() {
        assert!(supports_security_key_auth("ssh-ed25519\nsk-ssh-ed25519@openssh.com\n"));
        assert!(supports_security_key_auth("ecdsa-sk\n"));
        assert!(!supports_security_key_auth("ssh-ed25519\nrsa-sha2-512\n"));
    }
}
