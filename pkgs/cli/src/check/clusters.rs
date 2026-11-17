use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClusterId {
    Dtu,
    Gefion,
    Ngc,
}

pub const CLUSTER_IDS: [ClusterId; 3] = [ClusterId::Dtu, ClusterId::Gefion, ClusterId::Ngc];

impl ClusterId {
    pub fn as_str(self) -> &'static str {
        match self {
            ClusterId::Dtu => "dtu",
            ClusterId::Gefion => "gefion",
            ClusterId::Ngc => "ngc",
        }
    }

    pub fn parse(s: &str) -> Option<ClusterId> {
        CLUSTER_IDS.into_iter().find(|c| c.as_str() == s)
    }

    pub fn name(self) -> &'static str {
        match self {
            ClusterId::Dtu => DTU.name,
            ClusterId::Gefion => GEFION.name,
            ClusterId::Ngc => NGC.name,
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            ClusterId::Dtu => "DTU",
            ClusterId::Gefion => "Gefion",
            ClusterId::Ngc => "NGC",
        }
    }
}

impl std::fmt::Display for ClusterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub const SITE: &str = "https://hpc.aicentre.dk";

pub struct Dtu {
    pub name: &'static str,
    pub alias: &'static str,
    pub login: &'static str,
    pub port: u16,
    pub vpn: &'static str,
    pub project_dir: &'static str,
    pub docs: &'static str,
}

pub const DTU: Dtu = Dtu {
    name: "P1 DTU HPC",
    alias: "dtu",
    login: "login9.hpc.dtu.dk",
    port: 22,
    vpn: "vpn.dtu.dk",
    project_dir: "/dtu/p1",
    docs: "https://hpc.aicentre.dk/providers/dtu/",
};

pub struct Gefion {
    pub name: &'static str,
    pub alias: &'static str,
    pub login: &'static str,
    pub port: u16,
    pub proxy: &'static str,
    pub proxy_port: u16,
    pub docs: &'static str,
}

pub const GEFION: Gefion = Gefion {
    name: "P1 Gefion HPC",
    alias: "gefion",
    login: "slurm-login-01.hpc.ite.dcai.dk",
    port: 3022,
    proxy: "secureremote.dcai.dk",
    proxy_port: 443,
    docs: "https://hpc.aicentre.dk/providers/gefion/",
};

pub struct Ngc {
    pub name: &'static str,
    pub docs: &'static str,
}

pub const NGC: Ngc = Ngc { name: "P1 NGC HPC", docs: "https://hpc.aicentre.dk/providers/ngc/" };

pub fn teleport_key(tsh: &str, user: &str) -> String {
    format!("{tsh}/keys/{}/{user}", GEFION.proxy)
}

pub fn teleport_cert(tsh: &str, user: &str) -> String {
    format!("{tsh}/keys/{p}/{user}-ssh/{p}-cert.pub", p = GEFION.proxy)
}

pub const DTU_NO_ANSWER_HINT: &str = "login9.hpc.dtu.dk takes SSH from the internet. Check that this machine is online and that this network lets SSH (port 22) out; the DTU VPN (vpn.dtu.dk) gets around a network that does not.";

pub fn tsh_login(user: Option<&str>) -> String {
    match user {
        Some(u) => format!("tsh login --proxy {} --user {u}", GEFION.proxy),
        None => format!("tsh login --proxy {}", GEFION.proxy),
    }
}

pub fn host_block(cluster: ClusterId, alias: &str, user: Option<&str>) -> Vec<String> {
    if cluster == ClusterId::Dtu {
        return vec![
            format!("Host {alias}"),
            format!("  HostName {}", DTU.login),
            format!("  User {}", user.unwrap_or("<dtu-username>")),
            "  ServerAliveInterval 60".into(),
        ];
    }
    let u = user.unwrap_or("<gefion-username>");
    let p = GEFION.proxy;
    vec![
        format!("Host {alias}"),
        format!("  Port {}", GEFION.port),
        format!("  User {u}"),
        format!("  HostName {}", GEFION.login),
        format!(
            "  ProxyCommand tsh proxy ssh --cluster={p} --proxy={p}:{} %r@%h:%p",
            GEFION.proxy_port
        ),
        format!("  CertificateFile ~/.tsh/keys/{p}/{u}-ssh/{p}-cert.pub"),
        "  ConnectTimeout 30".into(),
        "  ControlMaster no".into(),
        "  ControlPersist no".into(),
        format!("  IdentityFile ~/.tsh/keys/{p}/{u}"),
        "  UserKnownHostsFile ~/.tsh/known_hosts".into(),
    ]
}

pub const GPU_HOST: &str = "aicentre-gpu";

pub struct GpuHost<'a> {
    pub user: &'a str,
    pub bin: &'a str,
    pub known_hosts: &'a str,
    pub identity_files: &'a [String],
}

pub fn gpu_host_block(o: &GpuHost) -> Vec<String> {
    let mut lines = vec![
        format!("Host {GPU_HOST} {GPU_HOST}-*"),
        format!("  User {}", o.user),
        format!("  ProxyCommand {} proxy %n", o.bin),
        format!("  UserKnownHostsFile {}", o.known_hosts),
        "  StrictHostKeyChecking yes".into(),
        "  ServerAliveInterval 30".into(),
    ];
    lines.extend(o.identity_files.iter().map(|f| format!("  IdentityFile {f}")));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_host_block_goes_through_aicentre_proxy() {
        let files = vec!["~/.ssh/dtu_key".to_string()];
        assert_eq!(
            gpu_host_block(&GpuHost {
                user: "s123456",
                bin: "~/.local/bin/aicentre",
                known_hosts: "~/.local/state/aicentre/known_hosts",
                identity_files: &files,
            }),
            [
                "Host aicentre-gpu aicentre-gpu-*",
                "  User s123456",
                "  ProxyCommand ~/.local/bin/aicentre proxy %n",
                "  UserKnownHostsFile ~/.local/state/aicentre/known_hosts",
                "  StrictHostKeyChecking yes",
                "  ServerAliveInterval 30",
                "  IdentityFile ~/.ssh/dtu_key",
            ]
        );
    }
}
