use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ColorType, DynamicImage, ImageEncoder, ImageFormat, Rgb, RgbImage, RgbaImage};
use shaft_core::normalize_format;

#[derive(Debug, Clone)]
pub struct ImageTranscodeOptions {
    pub quality: u8,
    pub strip_metadata: bool,
    pub background_color: [u8; 3],
    pub max_dimension: Option<u32>,
}

impl Default for ImageTranscodeOptions {
    fn default() -> Self {
        Self {
            quality: 85,
            strip_metadata: true,
            background_color: [255, 255, 255],
            max_dimension: None,
        }
    }
}

pub fn transcode_image<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
    options: &ImageTranscodeOptions,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();
    let norm_target = normalize_format(target_format);

    // 1. Decode source into DynamicImage
    let dynamic_img = decode_source(src)?;

    // 2. Optional resize if max_dimension is specified
    let processed_img = if let Some(max_dim) = options.max_dimension {
        if dynamic_img.width() > max_dim || dynamic_img.height() > max_dim {
            dynamic_img.resize(max_dim, max_dim, image::imageops::FilterType::Lanczos3)
        } else {
            dynamic_img
        }
    } else {
        dynamic_img
    };

    // 3. Encode to target format
    encode_target(&processed_img, dst, &norm_target, options)
}

fn decode_source(path: &Path) -> Result<DynamicImage, String> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    if ext == "svg" {
        return render_svg_to_image(path);
    }

    let reader = image::ImageReader::open(path)
        .map_err(|e| format!("Failed to open image file: {}", e))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to determine image format: {}", e))?;

    reader
        .decode()
        .map_err(|e| format!("Failed to decode image pixels: {}", e))
}

fn render_svg_to_image(path: &Path) -> Result<DynamicImage, String> {
    let svg_bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read SVG file: {}", e))?;

    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_bytes, &opt)
        .map_err(|e| format!("Failed to parse SVG tree: {}", e))?;

    let size = tree.size();
    let width = (size.width().ceil() as u32).max(1);
    let height = (size.height().ceil() as u32).max(1);

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to allocate memory for SVG pixmap".to_string())?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let rgba_img = RgbaImage::from_raw(width, height, pixmap.take())
        .ok_or_else(|| "Failed to construct RGBA image from rendered SVG".to_string())?;

    Ok(DynamicImage::ImageRgba8(rgba_img))
}

fn encode_target(
    img: &DynamicImage,
    dest: &Path,
    target_format: &str,
    options: &ImageTranscodeOptions,
) -> Result<(), String> {
    let file = File::create(dest)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(file);

    match target_format {
        "jpg" | "jpeg" => {
            // Flatten transparent alpha onto background matte
            let rgb_img = flatten_alpha_to_rgb(img, options.background_color);
            let encoder = JpegEncoder::new_with_quality(&mut writer, options.quality);
            encoder
                .write_image(
                    rgb_img.as_raw(),
                    rgb_img.width(),
                    rgb_img.height(),
                    ColorType::Rgb8.into(),
                )
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        }
        "png" => {
            let rgba = img.to_rgba8();
            let encoder = PngEncoder::new(&mut writer);
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ColorType::Rgba8.into(),
                )
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        }
        "webp" => {
            img.write_to(&mut writer, ImageFormat::WebP)
                .map_err(|e| format!("Failed to encode WebP: {}", e))?;
        }
        "bmp" => {
            let rgb_img = flatten_alpha_to_rgb(img, options.background_color);
            rgb_img
                .write_to(&mut writer, ImageFormat::Bmp)
                .map_err(|e| format!("Failed to encode BMP: {}", e))?;
        }
        "tiff" | "tif" => {
            img.write_to(&mut writer, ImageFormat::Tiff)
                .map_err(|e| format!("Failed to encode TIFF: {}", e))?;
        }
        "gif" => {
            img.write_to(&mut writer, ImageFormat::Gif)
                .map_err(|e| format!("Failed to encode GIF: {}", e))?;
        }
        "ico" => {
            let resized = if img.width() > 256 || img.height() > 256 {
                img.resize(256, 256, image::imageops::FilterType::Lanczos3)
            } else {
                img.clone()
            };
            resized
                .write_to(&mut writer, ImageFormat::Ico)
                .map_err(|e| format!("Failed to encode ICO: {}", e))?;
        }
        _ => {
            return Err(format!("Unsupported image target format: {}", target_format));
        }
    }

    Ok(())
}

fn flatten_alpha_to_rgb(img: &DynamicImage, bg: [u8; 3]) -> RgbImage {
    let rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    let mut rgb_img = RgbImage::new(width, height);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        if a == 255 {
            rgb_img.put_pixel(x, y, Rgb([r, g, b]));
        } else if a == 0 {
            rgb_img.put_pixel(x, y, Rgb(bg));
        } else {
            let alpha = a as u32;
            let inv_alpha = 255 - alpha;
            let blended_r = ((r as u32 * alpha + bg[0] as u32 * inv_alpha) / 255) as u8;
            let blended_g = ((g as u32 * alpha + bg[1] as u32 * inv_alpha) / 255) as u8;
            let blended_b = ((b as u32 * alpha + bg[2] as u32 * inv_alpha) / 255) as u8;
            rgb_img.put_pixel(x, y, Rgb([blended_r, blended_g, blended_b]));
        }
    }

    rgb_img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_to_jpeg_transcode_with_alpha_flatten() {
        let temp_dir = std::env::temp_dir();
        let src_png = temp_dir.join("shaft_unit_alpha.png");
        let dst_jpg = temp_dir.join("shaft_unit_alpha.jpg");

        // Create a 64x64 transparent RGBA image
        let mut rgba = RgbaImage::new(64, 64);
        for (x, _y, pixel) in rgba.enumerate_pixels_mut() {
            if x < 32 {
                *pixel = image::Rgba([255, 0, 0, 128]); // Semi-transparent red
            } else {
                *pixel = image::Rgba([0, 255, 0, 255]); // Solid green
            }
        }
        rgba.save(&src_png).unwrap();

        let opts = ImageTranscodeOptions::default();
        let res = transcode_image(&src_png, &dst_jpg, "jpg", &opts);
        assert!(res.is_ok(), "PNG to JPG transcode failed: {:?}", res);

        let decoded = image::open(&dst_jpg).unwrap();
        assert_eq!(decoded.width(), 64);
        assert_eq!(decoded.height(), 64);

        let _ = std::fs::remove_file(&src_png);
        let _ = std::fs::remove_file(&dst_jpg);
    }

    #[test]
    fn test_svg_to_png_vector_rasterization() {
        let temp_dir = std::env::temp_dir();
        let src_svg = temp_dir.join("shaft_test_vector.svg");
        let dst_png = temp_dir.join("shaft_test_vector.png");

        let svg_content = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
            <rect width="100" height="100" fill="#5e6ad2"/>
            <circle cx="50" cy="50" r="30" fill="#ffffff"/>
        </svg>"##;

        std::fs::write(&src_svg, svg_content).unwrap();

        let opts = ImageTranscodeOptions::default();
        let res = transcode_image(&src_svg, &dst_png, "png", &opts);
        assert!(res.is_ok(), "SVG to PNG vector transcode failed: {:?}", res);

        let decoded = image::open(&dst_png).unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 100);

        let _ = std::fs::remove_file(&src_svg);
        let _ = std::fs::remove_file(&dst_png);
    }

    #[test]
    fn test_png_to_webp_and_bmp() {
        let temp_dir = std::env::temp_dir();
        let src_png = temp_dir.join("shaft_unit_test.png");
        let dst_webp = temp_dir.join("shaft_unit_test.webp");
        let dst_bmp = temp_dir.join("shaft_unit_test.bmp");

        let mut rgba = RgbaImage::new(32, 32);
        for pixel in rgba.pixels_mut() {
            *pixel = image::Rgba([100, 150, 200, 255]);
        }
        rgba.save(&src_png).unwrap();

        let opts = ImageTranscodeOptions::default();
        assert!(transcode_image(&src_png, &dst_webp, "webp", &opts).is_ok());
        assert!(transcode_image(&src_png, &dst_bmp, "bmp", &opts).is_ok());

        assert!(dst_webp.exists());
        assert!(dst_bmp.exists());

        let _ = std::fs::remove_file(&src_png);
        let _ = std::fs::remove_file(&dst_webp);
        let _ = std::fs::remove_file(&dst_bmp);
    }
}
