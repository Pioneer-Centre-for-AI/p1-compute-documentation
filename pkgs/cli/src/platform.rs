#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Linux,
    Darwin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
}

impl Os {
    pub fn as_str(self) -> &'static str {
        match self {
            Os::Linux => "linux",
            Os::Darwin => "darwin",
        }
    }
}

impl Arch {
    pub fn as_str(self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

pub fn current_platform() -> Platform {
    Platform {
        os: if cfg!(target_os = "macos") { Os::Darwin } else { Os::Linux },
        arch: if cfg!(target_arch = "aarch64") { Arch::Arm64 } else { Arch::X64 },
    }
}

pub fn asset_name(p: Platform) -> String {
    format!("aicentre-{}-{}.tar.gz", p.os.as_str(), p.arch.as_str())
}

pub fn is_nix_store(path: &str) -> bool {
    path.starts_with("/nix/store/")
}
