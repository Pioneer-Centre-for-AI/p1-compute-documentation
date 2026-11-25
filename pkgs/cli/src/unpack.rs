use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use tar::{Archive, EntryType};

#[derive(Clone, Copy, Default)]
pub struct Rules<'a> {
    pub strip: usize,
    pub only: Option<&'a str>,
    pub under: Option<&'a str>,
    pub links: bool,
}

fn names(path: &Path) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for part in path.components() {
        match part {
            Component::Normal(name) => out.push(name.to_str()?.to_string()),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn link_stays_inside(at: &[String], target: &Path) -> bool {
    let mut depth = at.len().saturating_sub(1) as i64;
    for part in target.components() {
        match part {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => depth -= 1,
            _ => return false,
        }
        if depth < 0 {
            return false;
        }
    }
    true
}

/// No folder on the way to `rel` may be a link, or a file would be written
/// through it to somewhere else.
fn plain_folders(into: &Path, rel: &[String]) -> std::io::Result<PathBuf> {
    let mut dir = into.to_path_buf();
    for name in &rel[..rel.len() - 1] {
        dir.push(name);
        match std::fs::symlink_metadata(&dir) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(std::io::Error::other(format!("{} is in the way", dir.display()))),
            Err(_) => std::fs::DirBuilder::new().mode(0o755).create(&dir)?,
        }
    }
    Ok(dir.join(&rel[rel.len() - 1]))
}

fn clear(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => Ok(()),
        Ok(_) => std::fs::remove_file(path),
        Err(_) => Ok(()),
    }
}

pub fn unpack(gz: &[u8], into: &str, rules: Rules) -> Result<usize, String> {
    let into = Path::new(into);
    let mut archive = Archive::new(GzDecoder::new(gz));
    let mut files = 0;
    let broken = |e: std::io::Error| format!("the archive cannot be read: {e}");
    for entry in archive.entries().map_err(broken)? {
        let mut entry = entry.map_err(broken)?;
        let kind = entry.header().entry_type();
        if matches!(kind, EntryType::XGlobalHeader | EntryType::XHeader) {
            continue;
        }
        let path = entry.path().map_err(broken)?.into_owned();
        let shown = path.display().to_string();
        let Some(all) = names(&path) else {
            return Err(format!("the archive names {shown}, which is outside its folder"));
        };
        if all.len() <= rules.strip {
            continue;
        }
        let rel = &all[rules.strip..];
        if rules.only.is_some_and(|only| rel.join("/") != only)
            || rules.under.is_some_and(|top| rel[0] != top)
        {
            continue;
        }
        let write = |e: std::io::Error| format!("cannot unpack {shown}: {e}");
        match kind {
            EntryType::Directory => {
                let mut with_self = rel.to_vec();
                with_self.push(String::new());
                plain_folders(into, &with_self).map_err(write)?;
            }
            EntryType::Regular | EntryType::Continuous => {
                let to = plain_folders(into, rel).map_err(write)?;
                clear(&to).map_err(write)?;
                // Never setuid, and never writable by anyone else.
                let mode = entry.header().mode().unwrap_or(0o644) & 0o755;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(mode)
                    .open(&to)
                    .map_err(write)?;
                std::io::copy(&mut entry, &mut file).map_err(write)?;
                files += 1;
            }
            EntryType::Symlink if rules.links => {
                let target = entry.link_name().map_err(broken)?.map(|t| t.into_owned());
                let Some(target) = target.filter(|t| link_stays_inside(rel, t)) else {
                    return Err(format!("the archive links {shown} to outside its folder"));
                };
                let to = plain_folders(into, rel).map_err(write)?;
                clear(&to).map_err(write)?;
                std::os::unix::fs::symlink(target, &to).map_err(write)?;
            }
            _ => {
                let mut rest = Vec::new();
                let _ = entry.read_to_end(&mut rest);
                return Err(format!("the archive holds {shown}, which is not a plain file"));
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tar_gz(build: impl FnOnce(&mut tar::Builder<Vec<u8>>)) -> Vec<u8> {
        use std::io::Write;
        let mut builder = tar::Builder::new(Vec::new());
        build(&mut builder);
        let raw = builder.into_inner().unwrap();
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(&raw).unwrap();
        gz.finish().unwrap()
    }

    fn file(b: &mut tar::Builder<Vec<u8>>, path: &str, text: &str, mode: u32) {
        let mut header = tar::Header::new_gnu();
        header.set_size(text.len() as u64);
        header.set_mode(mode);
        header.set_entry_type(EntryType::Regular);
        // `append_data` refuses `..`, so the name is written as it is.
        let name = header.as_old_mut().name.as_mut_slice();
        name[..path.len()].copy_from_slice(path.as_bytes());
        header.set_cksum();
        b.append(&header, text.as_bytes()).unwrap();
    }

    fn link(b: &mut tar::Builder<Vec<u8>>, path: &str, target: &str) {
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_entry_type(EntryType::Symlink);
        header.set_path(path).unwrap();
        header.set_link_name(target).unwrap();
        header.set_cksum();
        b.append(&header, std::io::empty()).unwrap();
    }

    fn root() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().to_string_lossy().into_owned();
        (dir, at)
    }

    #[test]
    fn one_file_is_picked_and_loses_the_bits_that_give_rights_away() {
        use std::os::unix::fs::PermissionsExt;
        let gz = tar_gz(|b| {
            file(b, "aicentre", "bin", 0o4777);
            file(b, "extra", "x", 0o644);
        });
        let (dir, at) = root();
        let rules = Rules { only: Some("aicentre"), ..Default::default() };
        assert_eq!(unpack(&gz, &at, rules), Ok(1));
        assert!(!dir.path().join("extra").exists());
        let mode = std::fs::metadata(dir.path().join("aicentre")).unwrap().permissions().mode();
        assert_eq!(mode & 0o7777, 0o755);

        let linked = tar_gz(|b| link(b, "aicentre", "/bin/sh"));
        let err = unpack(&linked, &at, rules).unwrap_err();
        assert_eq!(err, "the archive holds aicentre, which is not a plain file");
    }

    #[test]
    fn nothing_is_written_outside_the_folder() {
        let (dir, at) = root();
        let inside = dir.path().join("in");
        std::fs::create_dir(&inside).unwrap();
        let into = inside.to_string_lossy().into_owned();
        let all = Rules { links: true, ..Default::default() };

        let up = tar_gz(|b| file(b, "../escaped", "x", 0o644));
        assert!(unpack(&up, &into, all).unwrap_err().contains("outside its folder"));
        let abs = tar_gz(|b| file(b, &format!("{at}/escaped"), "x", 0o644));
        assert!(unpack(&abs, &into, all).unwrap_err().contains("outside its folder"));
        for target in ["../../escaped", "/etc/passwd", "a/../../.."] {
            let out = tar_gz(|b| link(b, "d/l", target));
            assert!(unpack(&out, &into, all).unwrap_err().contains("to outside its folder"));
        }
        let through = tar_gz(|b| {
            link(b, "d", "..");
            file(b, "d/escaped", "x", 0o644);
        });
        assert!(unpack(&through, &into, all).is_err());
        assert!(!dir.path().join("escaped").exists());

        let fine = tar_gz(|b| {
            file(b, "top/bin/tool", "x", 0o755);
            link(b, "top/tool", "bin/tool");
            link(b, "top/bin/up", "../bin/tool");
        });
        let stripped = Rules { strip: 1, links: true, ..Default::default() };
        assert_eq!(unpack(&fine, &into, stripped), Ok(1));
        assert_eq!(std::fs::read_to_string(inside.join("tool")).unwrap(), "x");
        assert!(unpack(&fine, &into, Rules { strip: 1, ..Default::default() }).is_err());
    }

    #[test]
    fn only_one_top_folder_is_unpacked_when_asked() {
        let gz = tar_gz(|b| {
            file(b, "int-gpu/notebook.py", "x", 0o644);
            file(b, "other/planted", "x", 0o644);
            file(b, ".bashrc", "x", 0o644);
        });
        let (dir, at) = root();
        assert_eq!(unpack(&gz, &at, Rules { under: Some("int-gpu"), ..Default::default() }), Ok(1));
        let mut got: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        got.sort();
        assert_eq!(got, ["int-gpu"]);
        assert!(unpack(b"not gzip", &at, Rules::default()).is_err());
    }
}
