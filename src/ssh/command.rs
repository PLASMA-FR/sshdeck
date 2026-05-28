use std::process::Command;

use crate::ssh::host::SshHost;

pub fn ssh_args_for(host: &SshHost) -> Vec<String> {
    ssh_args_for_with_forwards(host, true)
}

pub fn ssh_noninteractive_args_for(host: &SshHost) -> Vec<String> {
    ssh_args_for_with_forwards(host, false)
}

fn ssh_args_for_with_forwards(host: &SshHost, include_forwards: bool) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(port) = host.port {
        args.extend(["-p".into(), port.to_string()]);
    }
    if let Some(identity) = &host.identity_file {
        args.extend(["-i".into(), identity.display().to_string()]);
    }
    if let Some(proxy_jump) = &host.proxy_jump {
        args.extend(["-J".into(), proxy_jump.clone()]);
    }
    if host
        .forward_agent
        .as_deref()
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "yes" | "true" | "on"))
    {
        args.push("-A".into());
    }
    if let Some(interval) = host.server_alive_interval {
        args.extend(["-o".into(), format!("ServerAliveInterval={interval}")]);
    }
    if include_forwards {
        for forward in &host.local_forwards {
            args.extend(["-L".into(), forward.clone()]);
        }
        for forward in &host.remote_forwards {
            args.extend(["-R".into(), forward.clone()]);
        }
    }

    args.push("--".into());
    args.push(ssh_destination_for(host));
    args
}

pub fn ssh_test_args_for(host: &SshHost, timeout_seconds: u64) -> Vec<String> {
    let mut args = vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        format!("ConnectTimeout={timeout_seconds}"),
    ];
    args.extend(ssh_args_for(host));
    args.push("exit".into());
    args
}

pub fn ssh_command_for(host: &SshHost) -> String {
    display_command("ssh", &ssh_args_for(host))
}

pub fn display_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| display_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_arg(arg: &str) -> String {
    if arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '@' | '=' | '~'))
    {
        arg.to_string()
    } else {
        shell_words::quote(arg).to_string()
    }
}

pub fn scp_download_command(host: &str, remote: &str, local: &str) -> String {
    format!(
        "scp -r {}:{} {}",
        shell_words::quote(host),
        shell_words::quote(remote),
        shell_words::quote(local)
    )
}

pub fn scp_upload_command(host: &str, local: &str, remote: &str) -> String {
    format!(
        "scp -r {} {}:{}",
        shell_words::quote(local),
        shell_words::quote(host),
        shell_words::quote(remote)
    )
}

pub fn is_dangerous_command(cmd: &str) -> bool {
    let c = cmd.to_ascii_lowercase();
    [
        "rm -rf",
        "mkfs",
        "dd ",
        "shutdown",
        "reboot",
        "passwd",
        "userdel",
        "iptables flush",
        "nft flush",
        "chmod -r 777",
        "chmod -r777",
        "chown -r",
    ]
    .iter()
    .any(|p| c.contains(p))
}

pub fn run_ssh_command(alias: &str, remote_command: &str) -> anyhow::Result<String> {
    let out = Command::new("ssh").arg("--").arg(alias).arg(remote_command).output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr))
}

fn ssh_destination_for(host: &SshHost) -> String {
    match (&host.user, &host.hostname) {
        (Some(user), Some(hostname)) if !user.trim().is_empty() => format!("{user}@{hostname}"),
        (_, Some(hostname)) => hostname.clone(),
        _ => host.alias.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn builds_ssh_command() {
        let h = SshHost {
            alias: "web prod".into(),
            ..Default::default()
        };
        assert_eq!(ssh_command_for(&h), "ssh -- 'web prod'");
    }

    #[test]
    fn managed_host_connection_uses_hostname_without_requiring_include_line() {
        let h = SshHost {
            alias: "web-prod-1".into(),
            hostname: Some("10.0.0.12".into()),
            user: Some("deploy".into()),
            port: Some(2222),
            identity_file: Some(PathBuf::from("~/.ssh/id_ed25519")),
            proxy_jump: Some("bastion".into()),
            forward_agent: Some("yes".into()),
            server_alive_interval: Some(30),
            ..Default::default()
        };

        let args = ssh_args_for(&h);
        assert_eq!(
            args,
            vec![
                "-p",
                "2222",
                "-i",
                "~/.ssh/id_ed25519",
                "-J",
                "bastion",
                "-A",
                "-o",
                "ServerAliveInterval=30",
                "--",
                "deploy@10.0.0.12",
            ]
        );
        assert_eq!(ssh_command_for(&h), "ssh -p 2222 -i ~/.ssh/id_ed25519 -J bastion -A -o ServerAliveInterval=30 -- deploy@10.0.0.12");
    }

    #[test]
    fn test_connection_args_add_batch_mode_and_timeout_before_destination() {
        let h = SshHost {
            alias: "pi".into(),
            hostname: Some("raspberrypi.local".into()),
            ..Default::default()
        };
        let args = ssh_test_args_for(&h, 5);
        assert_eq!(
            args,
            vec![
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
                "--",
                "raspberrypi.local",
                "exit"
            ]
        );
    }

    #[test]
    fn noninteractive_args_use_resolved_destination_without_opening_forwards() {
        let h = SshHost {
            alias: "server".into(),
            hostname: Some("10.0.0.44".into()),
            user: Some("ahmad".into()),
            port: Some(2222),
            local_forwards: vec!["8080 localhost:80".into()],
            remote_forwards: vec!["9090 localhost:90".into()],
            ..Default::default()
        };

        assert_eq!(
            ssh_noninteractive_args_for(&h),
            vec!["-p", "2222", "--", "ahmad@10.0.0.44"]
        );
    }


    #[test]
    fn destination_is_separated_from_options() {
        let h = SshHost {
            alias: "-oProxyCommand=evil".into(),
            ..Default::default()
        };
        let args = ssh_args_for(&h);
        assert_eq!(args, vec!["--", "-oProxyCommand=evil"]);
    }

    #[test]
    fn dangerous_recursive_permission_forms_are_detected() {
        assert!(is_dangerous_command("chmod -R777 /tmp"));
        assert!(is_dangerous_command("chmod -R 777 /tmp"));
    }

    #[test]
    fn detects_dangerous_commands() {
        assert!(is_dangerous_command("sudo rm -rf /var/www"));
        assert!(is_dangerous_command("reboot now"));
        assert!(!is_dangerous_command("uptime"));
    }

    #[test]
    fn quotes_scp_paths() {
        assert!(scp_upload_command("web", "/tmp/a b", "/var/www/a b").contains("'/tmp/a b'"));
    }
}
