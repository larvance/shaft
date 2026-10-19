use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::registry::{
    get_applicable_tools, get_available_targets, get_category_for_format, normalize_format,
};
use crate::types::{ExtendedMetadata, FileProbeResult};

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

    let (detected_format, ext_meta) = detect_format_and_metadata(path)?;
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
        metadata: ext_meta,
    })
}

fn detect_format_and_metadata(path: &Path) -> Result<(String, ExtendedMetadata), String> {
    let mut header = [0u8; 1024];
    let ext_fallback = path
        .extension()
        .map(|e| normalize_format(&e.to_string_lossy()))
        .unwrap_or_else(|| "unknown".to_string());

    let mut ext_meta = ExtendedMetadata::default();
    ext_meta.mime_type = mime_guess::from_path(path).first_raw().map(|s| s.to_string());

    let Ok(mut file) = File::open(path) else {
        return Ok((ext_fallback, ext_meta));
    };

    let bytes_read = file.read(&mut header).unwrap_or(0);
    if bytes_read >= 4 {
        // PNG
        if header.starts_with(b"\x89PNG\r\n\x1a\n") {
            if bytes_read >= 24 {
                let w = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
                let h = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
                ext_meta.width = Some(w);
                ext_meta.height = Some(h);
            }
            return Ok(("png".into(), ext_meta));
        }

        // JPEG
        if header.starts_with(b"\xFF\xD8\xFF") {
            // Scan for SOF markers (0xFF, 0xC0..0xC3)
            let mut i = 2;
            while i + 8 < bytes_read {
                if header[i] == 0xFF {
                    let marker = header[i + 1];
                    if (0xC0..=0xC3).contains(&marker) {
                        let h = u16::from_be_bytes([header[i + 5], header[i + 6]]) as u32;
                        let w = u16::from_be_bytes([header[i + 7], header[i + 8]]) as u32;
                        ext_meta.height = Some(h);
                        ext_meta.width = Some(w);
                        break;
                    }
                }
                i += 1;
            }
            return Ok(("jpg".into(), ext_meta));
        }

        // GIF
        if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
            if bytes_read >= 10 {
                let w = u16::from_le_bytes([header[6], header[7]]) as u32;
                let h = u16::from_le_bytes([header[8], header[9]]) as u32;
                ext_meta.width = Some(w);
                ext_meta.height = Some(h);
            }
            return Ok(("gif".into(), ext_meta));
        }

        // BMP
        if header.starts_with(b"BM") && bytes_read >= 26 {
            let w = i32::from_le_bytes([header[18], header[19], header[20], header[21]]).unsigned_abs();
            let h = i32::from_le_bytes([header[22], header[23], header[24], header[25]]).unsigned_abs();
            ext_meta.width = Some(w);
            ext_meta.height = Some(h);
            return Ok(("bmp".into(), ext_meta));
        }

        // PDF
        if header.starts_with(b"%PDF-") {
            ext_meta.page_count = Some(1); // Baseline probe
            return Ok(("pdf".into(), ext_meta));
        }

        // FLAC
        if header.starts_with(b"fLaC") {
            return Ok(("flac".into(), ext_meta));
        }

        // OGG / Opus
        if header.starts_with(b"OggS") {
            if ext_fallback == "opus" {
                return Ok(("opus".into(), ext_meta));
            }
            return Ok(("ogg".into(), ext_meta));
        }

        // WebP or WAV or AVI (RIFF container)
        if header.starts_with(b"RIFF") && bytes_read >= 12 {
            if &header[8..12] == b"WEBP" {
                return Ok(("webp".into(), ext_meta));
            }
            if &header[8..12] == b"WAVE" {
                if bytes_read >= 28 {
                    let channels = u16::from_le_bytes([header[22], header[23]]);
                    let sample_rate = u32::from_le_bytes([header[24], header[25], header[26], header[27]]);
                    ext_meta.channels = Some(channels);
                    ext_meta.sample_rate = Some(sample_rate);
                }
                return Ok(("wav".into(), ext_meta));
            }
            if &header[8..12] == b"AVI " {
                return Ok(("avi".into(), ext_meta));
            }
        }

        // Matroska / WebM
        if header.starts_with(b"\x1A\x45\xDF\xA3") {
            if ext_fallback == "webm" {
                return Ok(("webm".into(), ext_meta));
            }
            return Ok(("mkv".into(), ext_meta));
        }

        // MP4 / MOV (ftyp)
        if bytes_read >= 12 && &header[4..8] == b"ftyp" {
            let brand = &header[8..12];
            if brand == b"qt  " || ext_fallback == "mov" {
                return Ok(("mov".into(), ext_meta));
            }
            if brand == b"heic" || brand == b"mif1" || ext_fallback == "heic" {
                return Ok(("heic".into(), ext_meta));
            }
            if brand == b"avif" || ext_fallback == "avif" {
                return Ok(("avif".into(), ext_meta));
            }
            return Ok(("mp4".into(), ext_meta));
        }

        // MP3 (ID3 tag or sync frame)
        if header.starts_with(b"ID3") || (header[0] == 0xFF && (header[1] & 0xE0) == 0xE0) {
            return Ok(("mp3".into(), ext_meta));
        }

        // GZ
        if header[0] == 0x1F && header[1] == 0x8B {
            return Ok(("gz".into(), ext_meta));
        }

        // ZIP container (ZIP, DOCX, XLSX, PPTX, EPUB)
        if header.starts_with(b"PK\x03\x04") {
            match ext_fallback.as_str() {
                "docx" | "xlsx" | "pptx" | "epub" => return Ok((ext_fallback, ext_meta)),
                _ => return Ok(("zip".into(), ext_meta)),
            }
        }

        // RAR
        if header.starts_with(b"Rar!\x1A\x07") {
            return Ok(("rar".into(), ext_meta));
        }

        // TIFF
        if header.starts_with(b"II*\0") || header.starts_with(b"MM\0*") {
            return Ok(("tiff".into(), ext_meta));
        }
    }

    Ok((ext_fallback, ext_meta))
}
