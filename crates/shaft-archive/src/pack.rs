use std::fs::File;
use std::path::Path;
use flate2::write::GzEncoder;
use flate2::Compression;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn pack_archive<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    destination_archive: Q,
    target_format: &str,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = destination_archive.as_ref();

    if !src.exists() {
        return Err(format!("Source path does not exist: {}", src.display()));
    }

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let norm = shaft_core::normalize_format(target_format);

    match norm.as_str() {
        "zip" => pack_zip(src, dst),
        "tar" => pack_tar(src, dst),
        "gz" | "tgz" => pack_tar_gz(src, dst),
        _ => Err(format!("Unsupported archive packing format: {}", target_format)),
    }
}

fn pack_zip(src: &Path, dst: &Path) -> Result<(), String> {
    let file = File::create(dst).map_err(|e| format!("Failed to create ZIP: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    if src.is_file() {
        let file_name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        zip.start_file(file_name, options)
            .map_err(|e| format!("Failed to start ZIP entry: {}", e))?;
        let mut f = File::open(src).map_err(|e| format!("Failed to open file: {}", e))?;
        std::io::copy(&mut f, &mut zip).map_err(|e| format!("Failed to write to ZIP: {}", e))?;
    } else if src.is_dir() {
        let base_path = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
        for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = match path.strip_prefix(&base_path) {
                Ok(p) => p,
                Err(_) => match path.strip_prefix(src) {
                    Ok(p) => p,
                    Err(_) => continue,
                },
            };

            let rel_str = relative.to_string_lossy().replace('\\', "/");
            if rel_str.is_empty() {
                continue;
            }

            if entry.file_type().is_dir() {
                zip.add_directory(format!("{}/", rel_str), options)
                    .map_err(|e| format!("Failed to add directory to ZIP: {}", e))?;
            } else {
                zip.start_file(rel_str, options)
                    .map_err(|e| format!("Failed to add file to ZIP: {}", e))?;
                let mut f = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
                std::io::copy(&mut f, &mut zip)
                    .map_err(|e| format!("Failed to write file to ZIP: {}", e))?;
            }
        }
    }

    zip.finish().map_err(|e| format!("Failed to finalize ZIP: {}", e))?;
    Ok(())
}

fn pack_tar(src: &Path, dst: &Path) -> Result<(), String> {
    let file = File::create(dst).map_err(|e| format!("Failed to create TAR: {}", e))?;
    let mut builder = tar::Builder::new(file);

    if src.is_file() {
        let file_name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        let mut f = File::open(src).map_err(|e| format!("Failed to open file: {}", e))?;
        builder
            .append_file(&file_name, &mut f)
            .map_err(|e| format!("Failed to append to TAR: {}", e))?;
    } else if src.is_dir() {
        let dir_name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "archive".to_string());
        builder
            .append_dir_all(&dir_name, src)
            .map_err(|e| format!("Failed to append directory to TAR: {}", e))?;
    }

    builder.finish().map_err(|e| format!("Failed to finalize TAR: {}", e))?;
    Ok(())
}

fn pack_tar_gz(src: &Path, dst: &Path) -> Result<(), String> {
    let file = File::create(dst).map_err(|e| format!("Failed to create TAR.GZ: {}", e))?;
    let gz = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(gz);

    if src.is_file() {
        let file_name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        let mut f = File::open(src).map_err(|e| format!("Failed to open file: {}", e))?;
        builder
            .append_file(&file_name, &mut f)
            .map_err(|e| format!("Failed to append to TAR.GZ: {}", e))?;
    } else if src.is_dir() {
        let dir_name = src
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "archive".to_string());
        builder
            .append_dir_all(&dir_name, src)
            .map_err(|e| format!("Failed to append directory to TAR.GZ: {}", e))?;
    }

    builder.finish().map_err(|e| format!("Failed to finalize TAR.GZ: {}", e))?;
    Ok(())
}
