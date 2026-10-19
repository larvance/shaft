use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use image::codecs::ico::IcoEncoder;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};

pub fn lossless_compress_image<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source image does not exist: {}", s.display()));
    }

    let img = image::open(s).map_err(|e| format!("Failed to open image: {}", e))?;
    let rgba = img.to_rgba8();

    if let Some(parent) = d.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let file = File::create(d).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    // PNG with max compression and stripped metadata
    let encoder = PngEncoder::new(&mut writer);
    encoder
        .write_image(rgba.as_raw(), rgba.width(), rgba.height(), ColorType::Rgba8.into())
        .map_err(|e| format!("Failed to write compressed PNG: {}", e))?;

    Ok(())
}

pub fn convert_to_grayscale<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    let img = image::open(s).map_err(|e| format!("Failed to open image: {}", e))?;
    let gray = img.to_luma8();

    let file = File::create(d).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    let encoder = PngEncoder::new(&mut writer);
    encoder
        .write_image(gray.as_raw(), gray.width(), gray.height(), ColorType::L8.into())
        .map_err(|e| format!("Failed to write grayscale PNG: {}", e))?;

    Ok(())
}

pub fn generate_favicon_bundle<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    let img = image::open(s).map_err(|e| format!("Failed to open image: {}", e))?;
    let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);

    let file = File::create(d).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    let encoder = IcoEncoder::new(&mut writer);
    let rgba = resized.to_rgba8();

    encoder
        .write_image(rgba.as_raw(), rgba.width(), rgba.height(), ColorType::Rgba8.into())
        .map_err(|e| format!("Failed to write ICO favicon: {}", e))?;

    Ok(())
}

pub fn split_pdf_pages<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    output_dir: Q,
) -> Result<Vec<String>, String> {
    let s = src.as_ref();
    let d = output_dir.as_ref();

    if !s.exists() {
        return Err(format!("Source PDF does not exist: {}", s.display()));
    }

    if !d.exists() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }

    let doc = lopdf::Document::load(s)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;

    let pages = doc.get_pages();
    let mut created_paths = Vec::new();

    for (page_num, _page_id) in pages {
        let mut single_page_doc = doc.clone();
        let other_pages: Vec<u32> = single_page_doc
            .get_pages()
            .keys()
            .filter(|&&p| p != page_num)
            .copied()
            .collect();

        single_page_doc.delete_pages(&other_pages);

        let out_path = d.join(format!("page_{:03}.pdf", page_num));
        single_page_doc
            .save(&out_path)
            .map_err(|e| format!("Failed to save page {}: {}", page_num, e))?;

        created_paths.push(out_path.to_string_lossy().to_string());
    }

    Ok(created_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_lossless_compress_and_grayscale() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_img_in.png");
        let dst_comp = temp.join("shaft_img_comp.png");
        let dst_gray = temp.join("shaft_img_gray.png");

        let mut rgba = RgbaImage::new(64, 64);
        for pixel in rgba.pixels_mut() {
            *pixel = image::Rgba([120, 80, 200, 255]);
        }
        rgba.save(&src).unwrap();

        assert!(lossless_compress_image(&src, &dst_comp).is_ok());
        assert!(convert_to_grayscale(&src, &dst_gray).is_ok());

        assert!(dst_comp.exists());
        assert!(dst_gray.exists());

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst_comp);
        let _ = std::fs::remove_file(&dst_gray);
    }

    #[test]
    fn test_favicon_bundle() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_ico_in.png");
        let dst = temp.join("shaft_favicon.ico");

        let mut rgba = RgbaImage::new(32, 32);
        for pixel in rgba.pixels_mut() {
            *pixel = image::Rgba([50, 100, 150, 255]);
        }
        rgba.save(&src).unwrap();

        assert!(generate_favicon_bundle(&src, &dst).is_ok());
        assert!(dst.exists());
        assert!(std::fs::metadata(&dst).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    }
}
