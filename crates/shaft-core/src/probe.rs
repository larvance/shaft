use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::registry::{get_applicable_tools, get_available_targets, get_category_for_format, normalize_format};
use crate::types::FileProbeResult;

pub fn probe_file<P: AsRef<Path>>(path: P) -> Result<FileProbeResult, String> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let file_size_bytes = metadata.len();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let detected_format = detect_format(path)?;
    let category = get_category_for_format(&detected_format);
    let targets = get_available_targets(&detected_format)
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let tools = get_applicable_tools(category);

    Ok(FileProbeResult {
        file_path: path.to_string_lossy().to_string(),
        file_name,
        file_size_bytes,
        detected_format,
        category,
        available_targets: targets,
        applicable_tools: tools,
    })
}

fn detect_format(path: &Path) -> Result<String, String> {
    let mut header = [0u8; 64];
    let ext_fallback = path
        .extension()
        .map(|e| normalize_format(&e.to_string_lossy()))
        .unwrap_or_else(|| "unknown".to_string());

    let Ok(mut file) = File::open(path) else {
        return Ok(ext_fallback);
    };

    let bytes_read = file.read(&mut header).unwrap_or(0);
    if bytes_read >= 4 {
        // PNG
        if header.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Ok("png".into());
        }
        // JPEG
        if header.starts_with(b"\xFF\xD8\xFF") {
            return Ok("jpg".into());
        }
        // GIF
        if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
            return Ok("gif".into());
        }
        // PDF
        if header.starts_with(b"%PDF-") {
            return Ok("pdf".into());
        }
        // FLAC
        if header.starts_with(b"fLaC") {
            return Ok("flac".into());
        }
        // OGG / Opus
        if header.starts_with(b"OggS") {
            if ext_fallback == "opus" {
                return Ok("opus".into());
            }
            return Ok("ogg".into());
        }
        // WebP or WAV (RIFF container)
        if header.starts_with(b"RIFF") && bytes_read >= 12 {
            if &header[8..12] == b"WEBP" {
                return Ok("webp".into());
            }
            if &header[8..12] == b"WAVE" {
                return Ok("wav".into());
            }
            if &header[8..12] == b"AVI " {
                return Ok("avi".into());
            }
        }
        // Matroska / WebM
        if header.starts_with(b"\x1A\x45\xDF\xA3") {
            if ext_fallback == "webm" {
                return Ok("webm".into());
            }
            return Ok("mkv".into());
        }
        // MP4 / MOV (ftyp)
        if bytes_read >= 12 && &header[4..8] == b"ftyp" {
            let brand = &header[8..12];
            if brand == b"qt  " || ext_fallback == "mov" {
                return Ok("mov".into());
            }
            if brand == b"heic" || brand == b"mif1" || ext_fallback == "heic" {
                return Ok("heic".into());
            }
            if brand == b"avif" || ext_fallback == "avif" {
                return Ok("avif".into());
            }
            return Ok("mp4".into());
        }
        // MP3 (ID3 tag or sync frame)
        if header.starts_with(b"ID3") || (header[0] == 0xFF && (header[1] & 0xE0) == 0xE0) {
            return Ok("mp3".into());
        }
        // GZ
        if header[0] == 0x1F && header[1] == 0x8B {
            return Ok("gz".into());
        }
        // ZIP container (ZIP, DOCX, XLSX, PPTX, EPUB)
        if header.starts_with(b"PK\x03\x04") {
            match ext_fallback.as_str() {
                "docx" | "xlsx" | "pptx" | "epub" => return Ok(ext_fallback),
                _ => return Ok("zip".into()),
            }
        }
        // RAR
        if header.starts_with(b"Rar!\x1A\x07") {
            return Ok("rar".into());
        }
        // BMP
        if header.starts_with(b"BM") {
            return Ok("bmp".into());
        }
        // TIFF
        if header.starts_with(b"II*\0") || header.starts_with(b"MM\0*") {
            return Ok("tiff".into());
        }
    }

    Ok(ext_fallback)
}
