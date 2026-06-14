use serde::{Deserialize, Serialize};

use crate::ssh::command::display_command;

pub const TUNNEL_PROGRAM: &str = "ssh";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl TunnelCommand {
    pub fn display(&self) -> String {
        display_command(&self.program, &self.args)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TunnelType {
    Local,
    Remote,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelConfig {
    pub tunnel_type: TunnelType,
    pub host_alias: String,
    pub bind_address: Option<String>,
    pub local_port: u16,
    pub target_host: Option<String>,
    pub target_port: Option<u16>,
}

impl TunnelConfig {
    pub fn ssh_command(&self) -> TunnelCommand {
        TunnelCommand {
            program: TUNNEL_PROGRAM.into(),
            args: self.args(),
        }
    }

    pub fn args(&self) -> Vec<String> {
        let mut args = vec!["-N".into()];
        match self.tunnel_type {
            TunnelType::Local => args.extend(["-L".into(), self.forward_spec()]),
            TunnelType::Remote => args.extend(["-R".into(), self.forward_spec()]),
            TunnelType::Dynamic => args.extend(["-D".into(), self.dynamic_spec()]),
        }
        args.extend(["--".into(), self.host_alias.clone()]);
        args
    }

    pub fn command(&self) -> String {
        self.ssh_command().display()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.host_alias.trim().is_empty() {
            return Err("Pick a host before starting a tunnel.".into());
        }
        if self.local_port == 0 {
            return Err("Port 0 is not a usable tunnel port.".into());
        }
        if !matches!(self.tunnel_type, TunnelType::Dynamic) {
            if self.target_host.as_deref().unwrap_or("").trim().is_empty() {
                return Err("Target host is required for local and remote tunnels.".into());
            }
            if self.target_port.unwrap_or(0) == 0 {
                return Err("Target port is required for local and remote tunnels.".into());
            }
        }
        Ok(())
    }

    fn forward_spec(&self) -> String {
        format!(
            "{}{}:{}:{}",
            bind(&self.bind_address),
            self.local_port,
            self.target_host.clone().unwrap_or_else(|| "localhost".into()),
            self.target_port.unwrap_or(80)
        )
    }

    fn dynamic_spec(&self) -> String {
        format!("{}{}", bind(&self.bind_address), self.local_port)
    }
}

fn bind(b: &Option<String>) -> String {
    b.as_ref().map(|s| format!("{}:", s)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_tunnel_command() {
        let t = TunnelConfig {
            tunnel_type: TunnelType::Local,
            host_alias: "web-prod-1".into(),
            bind_address: None,
            local_port: 8080,
            target_host: Some("localhost".into()),
            target_port: Some(80),
        };
        assert_eq!(t.args(), vec!["-N", "-L", "8080:localhost:80", "--", "web-prod-1"]);
        assert_eq!(t.ssh_command().program, "ssh");
        assert_eq!(t.ssh_command().args, t.args());
        assert_eq!(t.ssh_command().display(), t.command());
        assert_eq!(t.command(), "ssh -N -L 8080:localhost:80 -- web-prod-1");
    }

    #[test]
    fn dynamic_tunnel_command() {
        let t = TunnelConfig {
            tunnel_type: TunnelType::Dynamic,
            host_alias: "web prod".into(),
            bind_address: Some("127.0.0.1".into()),
            local_port: 1080,
            target_host: None,
            target_port: None,
        };
        assert_eq!(t.args(), vec!["-N", "-D", "127.0.0.1:1080", "--", "web prod"]);
        assert_eq!(t.command(), "ssh -N -D 127.0.0.1:1080 -- 'web prod'");
    }

    #[test]
    fn validates_required_tunnel_fields() {
        let t = TunnelConfig {
            tunnel_type: TunnelType::Local,
            host_alias: String::new(),
            bind_address: None,
            local_port: 8080,
            target_host: Some("localhost".into()),
            target_port: Some(80),
        };
        assert!(t.validate().is_err());
    }
}
