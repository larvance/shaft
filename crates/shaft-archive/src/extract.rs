use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use flate2::read::GzDecoder;
use crate::sanitize::sanitize_archive_entry_path;

pub fn extract_archive<P: AsRef<Path>, Q: AsRef<Path>>(
    archive_path: P,
    destination_dir: Q,
) -> Result<(), String> {
    let src = archive_path.as_ref();
    let dst = destination_dir.as_ref();

    if !src.exists() {
        return Err(format!("Archive file does not exist: {}", src.display()));
    }

    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }

    let ext = src
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "zip" => extract_zip(src, dst),
        "tar" => extract_tar(src, dst),
        "gz" | "tgz" => extract_tar_gz(src, dst),
        _ => {
            // Check file signatures
            let mut magic = [0u8; 4];
            if let Ok(mut f) = File::open(src) {
                use std::io::Read;
                let _ = f.read_exact(&mut magic);
                if magic.starts_with(b"PK\x03\x04") {
                    return extract_zip(src, dst);
                }
                if magic.starts_with(&[0x1F, 0x8B]) {
                    return extract_tar_gz(src, dst);
                }
            }
            extract_tar(src, dst)
        }
    }
}

fn extract_zip(src: &Path, dst: &Path) -> Result<(), String> {
    let file = File::open(src).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let reader = BufReader::new(file);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry #{}: {}", i, e))?;

        let entry_name = file.name().map_err(|e| e.to_string())?;
        let target_path = sanitize_archive_entry_path(&entry_name, dst)?;

        if file.is_dir() {
            std::fs::create_dir_all(&target_path)
                .map_err(|e| format!("Failed to create directory {}: {}", target_path.display(), e))?;
        } else {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&target_path)
                .map_err(|e| format!("Failed to create file {}: {}", target_path.display(), e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract {}: {}", target_path.display(), e))?;
        }
    }

    Ok(())
}

fn extract_tar(src: &Path, dst: &Path) -> Result<(), String> {
    let file = File::open(src).map_err(|e| format!("Failed to open TAR: {}", e))?;
    let reader = BufReader::new(file);
    let mut archive = tar::Archive::new(reader);

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read TAR entries: {}", e))?;

    for entry_res in entries {
        let mut entry = entry_res.map_err(|e| format!("Invalid TAR entry: {}", e))?;
        let entry_path = entry
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let target_path = sanitize_archive_entry_path(&entry_path, dst)?;

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&target_path)
                .map_err(|e| format!("Failed to create directory {}: {}", target_path.display(), e))?;
        } else {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&target_path)
                .map_err(|e| format!("Failed to create file {}: {}", target_path.display(), e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to extract {}: {}", target_path.display(), e))?;
        }
    }

    Ok(())
}

fn extract_tar_gz(src: &Path, dst: &Path) -> Result<(), String> {
    let file = File::open(src).map_err(|e| format!("Failed to open TAR.GZ: {}", e))?;
    let reader = BufReader::new(file);
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read TAR.GZ entries: {}", e))?;

    for entry_res in entries {
        let mut entry = entry_res.map_err(|e| format!("Invalid TAR.GZ entry: {}", e))?;
        let entry_path = entry
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let target_path = sanitize_archive_entry_path(&entry_path, dst)?;

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&target_path)
                .map_err(|e| format!("Failed to create directory {}: {}", target_path.display(), e))?;
        } else {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&target_path)
                .map_err(|e| format!("Failed to create file {}: {}", target_path.display(), e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to extract {}: {}", target_path.display(), e))?;
        }
    }

    Ok(())
}
