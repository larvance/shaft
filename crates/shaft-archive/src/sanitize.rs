use std::path::{Component, Path, PathBuf};

pub fn sanitize_archive_entry_path(entry_path: &str, destination_dir: &Path) -> Result<PathBuf, String> {
    let p = Path::new(entry_path);
    let mut clean = PathBuf::new();

    for comp in p.components() {
        match comp {
            Component::Normal(c) => clean.push(c),
            Component::ParentDir => {
                return Err(format!(
                    "Illegal path traversal ('..') detected in archive entry: {}",
                    entry_path
                ));
            }
            Component::RootDir | Component::Prefix(_) | Component::CurDir => {}
        }
    }

    if clean.as_os_str().is_empty() {
        return Err("Archive entry resolved to empty path".into());
    }

    let full_path = destination_dir.join(&clean);

    // Canonical sandbox boundary check
    if let (Ok(canon_dest), Ok(canon_full)) = (destination_dir.canonicalize(), full_path.canonicalize()) {
        if !canon_full.starts_with(&canon_dest) {
            return Err(format!(
                "Zip slip escape blocked: path escapes destination directory ({})",
                canon_dest.display()
            ));
        }
    } else if !full_path.starts_with(destination_dir) {
        return Err(format!(
            "Zip slip escape blocked: path escapes destination directory ({})",
            destination_dir.display()
        ));
    }

    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizer_normal_path() {
        let dest = Path::new("/tmp/extract_target");
        let res = sanitize_archive_entry_path("docs/report.txt", dest);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), dest.join("docs/report.txt"));
    }

    #[test]
    fn test_sanitizer_blocks_zip_slip() {
        let dest = Path::new("/tmp/extract_target");
        let res1 = sanitize_archive_entry_path("../../etc/passwd", dest);
        assert!(res1.is_err());

        let res2 = sanitize_archive_entry_path("safe_dir/../../../var/log", dest);
        assert!(res2.is_err());
    }

    #[test]
    fn test_sanitizer_strips_leading_root() {
        let dest = Path::new("/tmp/extract_target");
        let res = sanitize_archive_entry_path("/etc/shadow", dest);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), dest.join("etc/shadow"));
    }
}
