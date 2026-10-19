use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntryInfo {
    pub path: String,
    pub is_dir: bool,
    pub uncompressed_size: u64,
    pub compressed_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    pub format: String,
    pub entry_count: usize,
    pub total_uncompressed_bytes: u64,
    pub total_compressed_bytes: u64,
    pub entries: Vec<ArchiveEntryInfo>,
}

pub fn inspect_archive<P: AsRef<Path>>(path: P) -> Result<ArchiveMetadata, String> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(format!("Archive file does not exist: {}", p.display()));
    }

    let ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    let file_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);

    match ext.as_str() {
        "zip" => inspect_zip(p, file_size),
        "tar" => inspect_tar(p, file_size),
        "gz" | "tgz" => inspect_tar_gz(p, file_size),
        _ => {
            // Check file signatures
            let mut magic = [0u8; 4];
            if let Ok(mut f) = File::open(p) {
                use std::io::Read;
                let _ = f.read_exact(&mut magic);
                if magic.starts_with(b"PK\x03\x04") {
                    return inspect_zip(p, file_size);
                }
                if magic.starts_with(&[0x1F, 0x8B]) {
                    return inspect_tar_gz(p, file_size);
                }
            }
            inspect_tar(p, file_size)
        }
    }
}

fn inspect_zip(path: &Path, file_size: u64) -> Result<ArchiveMetadata, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let reader = BufReader::new(file);
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    let mut entries = Vec::new();
    let mut total_uncompressed = 0u64;

    for i in 0..zip.len() {
        let entry = zip
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry #{}: {}", i, e))?;

        let is_dir = entry.is_dir();
        let size = entry.size();
        let comp_size = entry.compressed_size();
        total_uncompressed += size;

        let path_str = entry.name().map_err(|e| e.to_string())?.to_string();

        entries.push(ArchiveEntryInfo {
            path: path_str,
            is_dir,
            uncompressed_size: size,
            compressed_size: Some(comp_size),
        });
    }

    Ok(ArchiveMetadata {
        format: "zip".into(),
        entry_count: entries.len(),
        total_uncompressed_bytes: total_uncompressed,
        total_compressed_bytes: file_size,
        entries,
    })
}

fn inspect_tar(path: &Path, file_size: u64) -> Result<ArchiveMetadata, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open TAR: {}", e))?;
    let reader = BufReader::new(file);
    let mut archive = tar::Archive::new(reader);

    let mut entries = Vec::new();
    let mut total_uncompressed = 0u64;

    let tar_entries = archive
        .entries()
        .map_err(|e| format!("Failed to read TAR entries: {}", e))?;

    for entry_res in tar_entries {
        let entry = entry_res.map_err(|e| format!("Invalid TAR entry: {}", e))?;
        let path = entry
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let size = entry.size();
        let is_dir = entry.header().entry_type().is_dir();
        total_uncompressed += size;

        entries.push(ArchiveEntryInfo {
            path,
            is_dir,
            uncompressed_size: size,
            compressed_size: None,
        });
    }

    Ok(ArchiveMetadata {
        format: "tar".into(),
        entry_count: entries.len(),
        total_uncompressed_bytes: total_uncompressed,
        total_compressed_bytes: file_size,
        entries,
    })
}

fn inspect_tar_gz(path: &Path, file_size: u64) -> Result<ArchiveMetadata, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open TAR.GZ: {}", e))?;
    let reader = BufReader::new(file);
    let gz = GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz);

    let mut entries = Vec::new();
    let mut total_uncompressed = 0u64;

    let tar_entries = archive
        .entries()
        .map_err(|e| format!("Failed to read TAR.GZ entries: {}", e))?;

    for entry_res in tar_entries {
        let entry = entry_res.map_err(|e| format!("Invalid TAR.GZ entry: {}", e))?;
        let path = entry
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let size = entry.size();
        let is_dir = entry.header().entry_type().is_dir();
        total_uncompressed += size;

        entries.push(ArchiveEntryInfo {
            path,
            is_dir,
            uncompressed_size: size,
            compressed_size: None,
        });
    }

    Ok(ArchiveMetadata {
        format: "tar.gz".into(),
        entry_count: entries.len(),
        total_uncompressed_bytes: total_uncompressed,
        total_compressed_bytes: file_size,
        entries,
    })
}
