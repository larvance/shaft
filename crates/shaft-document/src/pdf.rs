use std::path::Path;
use shaft_core::normalize_format;

pub fn extract_text_from_pdf<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let p = path.as_ref();
    if !p.exists() {
        return Err(format!("PDF file does not exist: {}", p.display()));
    }

    let doc = lopdf::Document::load(p)
        .map_err(|e| format!("Failed to parse PDF document: {}", e))?;

    let pages = doc.get_pages();
    let mut page_keys: Vec<u32> = pages.keys().copied().collect();
    page_keys.sort_unstable();

    let mut full_text = String::new();

    for page_num in page_keys {
        let text = doc.extract_text(&[page_num]).unwrap_or_default();
        if !text.trim().is_empty() {
            full_text.push_str(&format!("--- Page {} ---\n\n", page_num));
            full_text.push_str(&text);
            full_text.push_str("\n\n");
        }
    }

    Ok(full_text.trim().to_string())
}

pub fn transcode_pdf<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    let text = extract_text_from_pdf(src)?;
    let norm = normalize_format(target_format);

    let output_str = match norm.as_str() {
        "txt" => text,
        "md" => {
            format!("# PDF Export\n\n{}", text)
        }
        "html" | "htm" => {
            format!(
                "<!DOCTYPE html>\n<html>\n<head><meta charset=\"utf-8\"><title>PDF Export</title></head>\n<body>\n<pre style=\"white-space: pre-wrap; font-family: monospace;\">\n{}\n</pre>\n</body>\n</html>\n",
                html_escape(&text)
            )
        }
        _ => text,
    };

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    std::fs::write(dst, output_str).map_err(|e| e.to_string())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_non_existent_source() {
        let non_existent = Path::new("/path/to/missing/file.pdf");
        let dst = Path::new("/tmp/should_fail.txt");
        let res = transcode_pdf(non_existent, dst, "txt");
        assert!(res.is_err());
    }
}
