use std::path::Path;
use shaft_core::normalize_format;
use crate::extract::extract_archive;
use crate::pack::pack_archive;

pub fn transcode_archive<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Source archive does not exist: {}", src.display()));
    }

    let norm = normalize_format(target_format);

    // If target is uncompressed directory extract
    if norm == "unpacked" || norm == "dir" || norm == "folder" {
        return extract_archive(src, dst);
    }

    // Convert archive formats (e.g. tar -> zip, zip -> tar.gz)
    let temp_unpack = std::env::temp_dir().join(format!(
        "shaft_unpack_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));

    std::fs::create_dir_all(&temp_unpack)
        .map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    // 1. Extract source archive
    let extract_res = extract_archive(src, &temp_unpack);
    if let Err(e) = extract_res {
        let _ = std::fs::remove_dir_all(&temp_unpack);
        return Err(format!("Failed to unpack source archive: {}", e));
    }

    // 2. Pack to destination format
    let pack_res = pack_archive(&temp_unpack, dst, &norm);

    // 3. Clean up temporary directory
    let _ = std::fs::remove_dir_all(&temp_unpack);

    pack_res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::inspect_archive;

    #[test]
    fn test_zip_pack_inspect_and_extract() {
        let temp_dir = std::env::temp_dir();
        let test_root = temp_dir.join("shaft_archive_unit_test");
        let _ = std::fs::remove_dir_all(&test_root);
        std::fs::create_dir_all(&test_root).unwrap();

        // Create sample folder structure
        let sub_dir = test_root.join("subfolder");
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(test_root.join("hello.txt"), "Hello Shaft Engine\n").unwrap();
        std::fs::write(sub_dir.join("telemetry.json"), r#"{"status":"nominal"}"#).unwrap();

        let dst_zip = temp_dir.join("shaft_output.zip");
        let _ = std::fs::remove_file(&dst_zip);

        // 1. Pack directory into ZIP
        let pack_res = pack_archive(&test_root, &dst_zip, "zip");
        assert!(pack_res.is_ok(), "Packing ZIP failed: {:?}", pack_res);
        assert!(dst_zip.exists());

        // 2. In-Memory Inspect
        let meta = inspect_archive(&dst_zip).unwrap();
        assert_eq!(meta.format, "zip");
        assert!(meta.entry_count >= 2);
        assert!(meta.entries.iter().any(|e| e.path.contains("hello.txt")));
        assert!(meta.entries.iter().any(|e| e.path.contains("telemetry.json")));

        // 3. Extract to another folder
        let extract_target = temp_dir.join("shaft_extracted_test");
        let _ = std::fs::remove_dir_all(&extract_target);
        let ext_res = extract_archive(&dst_zip, &extract_target);
        assert!(ext_res.is_ok(), "Extraction failed: {:?}", ext_res);
        assert!(extract_target.join("hello.txt").exists());
        assert!(extract_target.join("subfolder").join("telemetry.json").exists());

        // 4. Transcode ZIP -> TAR
        let dst_tar = temp_dir.join("shaft_transcoded.tar");
        let _ = std::fs::remove_file(&dst_tar);
        let trans_res = transcode_archive(&dst_zip, &dst_tar, "tar");
        assert!(trans_res.is_ok(), "Transcode to TAR failed: {:?}", trans_res);
        assert!(dst_tar.exists());
        assert!(std::fs::metadata(&dst_tar).unwrap().len() > 0);

        // Clean up
        let _ = std::fs::remove_dir_all(&test_root);
        let _ = std::fs::remove_dir_all(&extract_target);
        let _ = std::fs::remove_file(&dst_zip);
        let _ = std::fs::remove_file(&dst_tar);
    }
}
