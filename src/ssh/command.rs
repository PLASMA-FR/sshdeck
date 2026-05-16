use std::process::Command;
use crate::ssh::host::SshHost;

pub fn ssh_command_for(host: &SshHost) -> String { format!("ssh {}", shell_words::quote(&host.alias)) }
pub fn scp_download_command(host: &str, remote: &str, local: &str) -> String { format!("scp -r {}:{} {}", shell_words::quote(host), shell_words::quote(remote), shell_words::quote(local)) }
pub fn scp_upload_command(host: &str, local: &str, remote: &str) -> String { format!("scp -r {} {}:{}", shell_words::quote(local), shell_words::quote(host), shell_words::quote(remote)) }
pub fn is_dangerous_command(cmd: &str) -> bool { let c=cmd.to_ascii_lowercase(); ["rm -rf", "mkfs", "dd ", "shutdown", "reboot", "passwd", "userdel", "iptables flush", "nft flush", "chmod -r 777", "chown -r"].iter().any(|p| c.contains(p)) }
pub fn run_ssh_command(alias: &str, remote_command: &str) -> anyhow::Result<String> { let out = Command::new("ssh").arg(alias).arg(remote_command).output()?; Ok(String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)) }
#[cfg(test)] mod tests { use super::*; #[test] fn builds_ssh_command(){ let h=SshHost{alias:"web prod".into(),..Default::default()}; assert_eq!(ssh_command_for(&h), "ssh 'web prod'"); } #[test] fn detects_dangerous_commands(){ assert!(is_dangerous_command("sudo rm -rf /var/www")); assert!(is_dangerous_command("reboot now")); assert!(!is_dangerous_command("uptime")); } #[test] fn quotes_scp_paths(){ assert!(scp_upload_command("web", "/tmp/a b", "/var/www/a b").contains("'/tmp/a b'")); } }
