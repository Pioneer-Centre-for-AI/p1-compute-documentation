use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;

pub fn stat_or_null(path: &str) -> Option<Metadata> {
    std::fs::metadata(path).ok()
}

pub fn octal(mode: u32) -> String {
    format!("{:04o}", mode & 0o777)
}

pub fn config_problem(st: &Metadata, uid: u32) -> Option<String> {
    if st.uid() != uid && st.uid() != 0 {
        return Some("belongs to another user".into());
    }
    if st.mode() & 0o022 != 0 {
        return Some(format!("others can write to it (mode {})", octal(st.mode())));
    }
    None
}

pub fn private_key_problem(st: &Metadata, uid: u32) -> Option<String> {
    (st.uid() == uid && st.mode() & 0o077 != 0)
        .then(|| format!("mode {} is too open, so ssh ignores it", octal(st.mode())))
}

pub fn ssh_dir_problem(st: &Metadata, uid: u32) -> Option<String> {
    if !st.is_dir() {
        return Some("is not a directory".into());
    }
    if st.uid() != uid {
        return Some("belongs to another user".into());
    }
    if st.mode() & 0o077 != 0 {
        return Some(format!("mode {}; 0700 keeps it private to you", octal(st.mode())));
    }
    None
}
