use std::{
    io::{self, Read},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::ssh::host::SshHost;

pub const DEFAULT_REMOTE_COMMAND_TIMEOUT: Duration = Duration::from_secs(20);
pub const DEFAULT_REMOTE_COMMAND_OUTPUT_LIMIT: usize = 64 * 1024;

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

pub fn scp_download_args_for(host: &SshHost, remote: &str, local: &str) -> Vec<String> {
    let mut args = scp_base_args_for(host);
    args.extend([
        "-r".into(),
        "--".into(),
        format!("{}:{}", ssh_destination_for(host), remote),
        local.into(),
    ]);
    args
}

pub fn scp_upload_args_for(host: &SshHost, local: &str, remote: &str) -> Vec<String> {
    let mut args = scp_base_args_for(host);
    args.extend([
        "-r".into(),
        "--".into(),
        local.into(),
        format!("{}:{}", ssh_destination_for(host), remote),
    ]);
    args
}

fn scp_base_args_for(host: &SshHost) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(port) = host.port {
        args.extend(["-P".into(), port.to_string()]);
    }
    if let Some(identity) = &host.identity_file {
        args.extend(["-i".into(), identity.display().to_string()]);
    }
    if let Some(proxy_jump) = &host.proxy_jump {
        args.extend(["-J".into(), proxy_jump.clone()]);
    }
    args
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
    let host = SshHost {
        alias: alias.into(),
        ..Default::default()
    };
    run_ssh_command_for(
        &host,
        remote_command,
        DEFAULT_REMOTE_COMMAND_TIMEOUT,
        DEFAULT_REMOTE_COMMAND_OUTPUT_LIMIT,
    )
}

pub fn ssh_remote_command_args_for(host: &SshHost, remote_command: &str) -> Vec<String> {
    let mut args = ssh_noninteractive_args_for(host);
    args.push(remote_command.into());
    args
}

pub fn run_ssh_command_for(
    host: &SshHost,
    remote_command: &str,
    timeout: Duration,
    max_bytes: usize,
) -> anyhow::Result<String> {
    let remaining = Arc::new(AtomicUsize::new(max_bytes));
    let truncated = Arc::new(AtomicBool::new(false));
    let mut child = Command::new("ssh")
        .args(ssh_remote_command_args_for(host, remote_command))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_handle = capture_capped_output(stdout, Arc::clone(&remaining), Arc::clone(&truncated));
    let stderr_handle = capture_capped_output(stderr, remaining, Arc::clone(&truncated));
    let started = Instant::now();
    let mut timed_out = false;

    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let stdout = join_captured_output(stdout_handle)?;
    let stderr = join_captured_output(stderr_handle)?;
    let output = capped_output(&stdout, &stderr, truncated.load(Ordering::Relaxed));
    if timed_out {
        anyhow::bail!(
            "remote command timed out after {}s\n{}",
            timeout.as_secs(),
            output
        );
    }
    Ok(output)
}

fn capture_capped_output<R>(
    mut reader: R,
    remaining: Arc<AtomicUsize>,
    truncated: Arc<AtomicBool>,
) -> JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = Vec::new();
        let mut buffer = [0_u8; 8192];
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            };
            let keep = reserve_output_bytes(&remaining, read);
            if keep < read {
                truncated.store(true, Ordering::Relaxed);
            }
            output.extend_from_slice(&buffer[..keep]);
        }
        Ok(output)
    })
}

fn reserve_output_bytes(remaining: &AtomicUsize, requested: usize) -> usize {
    loop {
        let available = remaining.load(Ordering::Relaxed);
        if available == 0 {
            return 0;
        }
        let keep = available.min(requested);
        if remaining
            .compare_exchange(available, available - keep, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            return keep;
        }
    }
}

fn join_captured_output(handle: JoinHandle<io::Result<Vec<u8>>>) -> anyhow::Result<Vec<u8>> {
    handle.join().map_err(|_| anyhow::anyhow!("remote command output reader panicked"))?
        .map_err(Into::into)
}

fn capped_output(stdout: &[u8], stderr: &[u8], truncated: bool) -> String {
    let mut bytes = Vec::with_capacity(stdout.len().saturating_add(stderr.len()));
    bytes.extend_from_slice(stdout);
    if !stderr.is_empty() {
        if !bytes.ends_with(b"\n") && !bytes.is_empty() {
            bytes.push(b'\n');
        }
        bytes.extend_from_slice(stderr);
    }
    let mut text = String::from_utf8_lossy(&bytes).to_string();
    if truncated {
        text.push_str("\n[output truncated]");
    }
    text
}

pub fn ssh_destination_for(host: &SshHost) -> String {
    match (&host.user, &host.hostname) {
        (Some(user), Some(hostname)) if !user.trim().is_empty() => format!("{user}@{hostname}"),
        (_, Some(hostname)) => hostname.clone(),
        _ => host.alias.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        sync::{Mutex, MutexGuard, OnceLock},
    };

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct PathGuard {
        original: Option<OsString>,
    }

    impl PathGuard {
        fn prepend(path: &Path) -> Self {
            let original = env::var_os("PATH");
            let mut paths = vec![path.to_path_buf()];
            if let Some(existing) = &original {
                paths.extend(env::split_paths(existing));
            }
            let joined = env::join_paths(paths).unwrap();
            // SAFETY: this test-only mutation is scoped by env_lock and restored
            // in Drop. The fake helper also refuses unrelated invocations.
            unsafe { env::set_var("PATH", joined) };
            Self { original }
        }
    }

    impl Drop for PathGuard {
        fn drop(&mut self) {
            // SAFETY: this restores the process PATH changed while env_lock is held.
            unsafe {
                match &self.original {
                    Some(path) => env::set_var("PATH", path),
                    None => env::remove_var("PATH"),
                }
            }
        }
    }

    #[cfg(unix)]
    fn write_fake_ssh(dir: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let ssh = dir.join("ssh");
        fs::write(
            &ssh,
            r#"#!/bin/sh
i=0
for arg in "$@"; do
  i=$((i + 1))
  printf 'arg%s=%s\n' "$i" "$arg"
done
printf 'fake ssh stderr\n' >&2
for arg in "$@"; do
  if [ "$arg" = "sshdeck-fake-ok" ]; then
    exit 0
  fi
done
printf 'fake ssh refused\n' >&2
exit 255
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&ssh).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(ssh, permissions).unwrap();
    }

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
    fn remote_command_args_append_command_after_separated_destination() {
        let h = SshHost {
            alias: "-oProxyCommand=evil".into(),
            ..Default::default()
        };

        assert_eq!(
            ssh_remote_command_args_for(&h, "uptime"),
            vec!["--", "-oProxyCommand=evil", "uptime"]
        );
    }

    #[test]
    fn capped_output_marks_truncation() {
        assert_eq!(capped_output(b"abcd", b"", true), "abcd\n[output truncated]");
    }

    #[test]
    fn capped_output_keeps_stderr_separated_from_stdout() {
        assert_eq!(capped_output(b"stdout", b"stderr", false), "stdout\nstderr");
        assert_eq!(capped_output(b"", b"stderr", false), "stderr");
    }

    #[cfg(unix)]
    #[test]
    fn run_ssh_command_uses_fake_openssh_helper_without_network() {
        let _env = env_lock();
        let dir = tempfile::tempdir().unwrap();
        write_fake_ssh(dir.path());
        let _path = PathGuard::prepend(dir.path());

        let alias_output = run_ssh_command("fake-host", "sshdeck-fake-ok").unwrap();
        assert!(alias_output.contains("arg1=--"));
        assert!(alias_output.contains("arg2=fake-host"));
        assert!(alias_output.contains("arg3=sshdeck-fake-ok"));
        assert!(alias_output.contains("fake ssh stderr"));

        let host = SshHost {
            alias: "ignored-alias".into(),
            hostname: Some("10.0.0.2".into()),
            user: Some("deploy".into()),
            port: Some(2222),
            ..Default::default()
        };
        let host_output = run_ssh_command_for(
            &host,
            "sshdeck-fake-ok",
            std::time::Duration::from_secs(1),
            4096,
        )
        .unwrap();
        assert!(host_output.contains("arg1=-p"));
        assert!(host_output.contains("arg2=2222"));
        assert!(host_output.contains("arg3=--"));
        assert!(host_output.contains("arg4=deploy@10.0.0.2"));
        assert!(host_output.contains("arg5=sshdeck-fake-ok"));
        assert!(host_output.contains("fake ssh stderr"));
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

    #[test]
    fn builds_scp_args_with_destination_separator() {
        let h = SshHost {
            alias: "-oProxyCommand=evil".into(),
            hostname: Some("10.0.0.2".into()),
            user: Some("deploy".into()),
            port: Some(2222),
            proxy_jump: Some("bastion".into()),
            ..Default::default()
        };

        assert_eq!(
            scp_download_args_for(&h, "/tmp/a b", "/tmp/out"),
            vec![
                "-P",
                "2222",
                "-J",
                "bastion",
                "-r",
                "--",
                "deploy@10.0.0.2:/tmp/a b",
                "/tmp/out",
            ]
        );
    }

    #[test]
    fn builds_scp_upload_args_with_identity_and_destination_separator() {
        let h = SshHost {
            alias: "web-prod".into(),
            hostname: Some("10.0.0.2".into()),
            user: Some("deploy".into()),
            identity_file: Some(PathBuf::from("/tmp/key with space")),
            ..Default::default()
        };

        assert_eq!(
            scp_upload_args_for(&h, "-local-file", "/srv/app config"),
            vec![
                "-i",
                "/tmp/key with space",
                "-r",
                "--",
                "-local-file",
                "deploy@10.0.0.2:/srv/app config",
            ]
        );
    }
}
