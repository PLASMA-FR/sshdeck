use serde::{Deserialize, Serialize};

use crate::ssh::{command::ssh_destination_for, host::SshHost};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferDirection {
    Upload,
    Download,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferStatus {
    Queued,
    Active,
    Done,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferJob {
    pub id: u64,
    pub direction: TransferDirection,
    pub source: String,
    pub destination: String,
    pub progress: u8,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferCommand {
    pub program: &'static str,
    pub args: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct TransferQueue {
    pub jobs: Vec<TransferJob>,
    next_id: u64,
}

impl TransferQueue {
    pub fn enqueue(
        &mut self,
        direction: TransferDirection,
        source: String,
        destination: String,
    ) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.jobs.push(TransferJob {
            id,
            direction,
            source,
            destination,
            progress: 0,
            status: TransferStatus::Queued,
        });
        id
    }

    pub fn start_next(&mut self) {
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|job| job.status == TransferStatus::Queued)
        {
            job.status = TransferStatus::Active;
        }
    }

    pub fn start(&mut self, id: u64) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id == id) {
            job.status = TransferStatus::Active;
            job.progress = job.progress.min(99);
        }
    }

    pub fn complete(&mut self, id: u64) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id == id) {
            job.progress = 100;
            job.status = TransferStatus::Done;
        }
    }

    pub fn fail(&mut self, id: u64, msg: String) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id == id) {
            job.status = TransferStatus::Failed(msg);
        }
    }

    pub fn active_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|job| matches!(job.status, TransferStatus::Queued | TransferStatus::Active))
            .count()
    }
}

pub fn transfer_command_for(
    host: &SshHost,
    direction: TransferDirection,
    source: &str,
    destination: &str,
) -> TransferCommand {
    TransferCommand {
        program: "scp",
        args: transfer_args_for(host, direction, source, destination),
    }
}

pub fn transfer_args_for(
    host: &SshHost,
    direction: TransferDirection,
    source: &str,
    destination: &str,
) -> Vec<String> {
    match direction {
        TransferDirection::Upload => scp_upload_args_for(host, source, destination),
        TransferDirection::Download => scp_download_args_for(host, source, destination),
    }
}

pub fn scp_download_args_for(host: &SshHost, remote: &str, local: &str) -> Vec<String> {
    let mut args = scp_base_args_for(host);
    args.extend([
        "-r".into(),
        "--".into(),
        remote_operand(host, remote),
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
        remote_operand(host, remote),
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

fn remote_operand(host: &SshHost, path: &str) -> String {
    format!("{}:{path}", ssh_destination_for(host))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn transfer_queue_state_transitions() {
        let mut queue = TransferQueue::default();
        let id = queue.enqueue(TransferDirection::Upload, "a".into(), "b".into());
        assert_eq!(queue.active_count(), 1);
        queue.start_next();
        assert_eq!(queue.jobs[0].status, TransferStatus::Active);
        queue.complete(id);
        assert_eq!(queue.jobs[0].status, TransferStatus::Done);
        assert_eq!(queue.active_count(), 0);
    }

    #[test]
    fn transfer_queue_can_start_specific_job() {
        let mut queue = TransferQueue::default();
        let first = queue.enqueue(TransferDirection::Upload, "a".into(), "b".into());
        let second = queue.enqueue(TransferDirection::Download, "c".into(), "d".into());

        queue.start(second);

        assert_eq!(queue.jobs.iter().find(|job| job.id == first).unwrap().status, TransferStatus::Queued);
        assert_eq!(queue.jobs.iter().find(|job| job.id == second).unwrap().status, TransferStatus::Active);
    }

    #[test]
    fn builds_download_args_with_destination_separator() {
        let host = SshHost {
            alias: "-oProxyCommand=evil".into(),
            hostname: Some("10.0.0.2".into()),
            user: Some("deploy".into()),
            port: Some(2222),
            identity_file: Some(PathBuf::from("~/.ssh/id_ed25519")),
            proxy_jump: Some("bastion".into()),
            ..Default::default()
        };

        assert_eq!(
            scp_download_args_for(&host, "/tmp/a b", "/tmp/out"),
            vec![
                "-P",
                "2222",
                "-i",
                "~/.ssh/id_ed25519",
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
    fn builds_upload_transfer_command() {
        let host = SshHost {
            alias: "web".into(),
            ..Default::default()
        };

        assert_eq!(
            transfer_command_for(&host, TransferDirection::Upload, "/tmp/local", "~/remote"),
            TransferCommand {
                program: "scp",
                args: vec![
                    "-r".into(),
                    "--".into(),
                    "/tmp/local".into(),
                    "web:~/remote".into(),
                ],
            }
        );
    }
}
