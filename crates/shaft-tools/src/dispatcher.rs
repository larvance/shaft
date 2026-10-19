use std::path::Path;
use serde_json::Value;
use crate::actions::{copy_file, reveal_in_terminal};
use crate::audio::{generate_waveform_video, normalize_loudness, optimize_audio_bitrate, trim_silence};
use crate::image_pdf::{convert_to_grayscale, generate_favicon_bundle, lossless_compress_image, split_pdf_pages};
use crate::video::{adjust_video_speed, crop_aspect_ratio, downscale_video, extract_video_frames};

pub fn execute_tool(
    file_path: &str,
    tool_id: &str,
    options: &Value,
) -> Result<String, String> {
    let src = Path::new(file_path);
    if !src.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    let parent = src.parent().unwrap_or_else(|| Path::new("."));
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");

    match tool_id {
        // Audio tools
        "audio_silence_trim" => {
            let out_path = parent.join(format!("{}_trimmed.{}", stem, ext));
            trim_silence(src, &out_path)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "loudness_normalize" => {
            let out_path = parent.join(format!("{}_normalized.{}", stem, ext));
            normalize_loudness(src, &out_path)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "waveform_video" => {
            let out_path = parent.join(format!("{}_waveform.mp4", stem));
            generate_waveform_video(src, &out_path)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "audio_bitrate_optimize" => {
            let bitrate = options.get("bitrate_kbps").and_then(|v| v.as_u64()).unwrap_or(192) as u32;
            let out_path = parent.join(format!("{}_opt.{}", stem, ext));
            optimize_audio_bitrate(src, &out_path, bitrate)?;
            Ok(out_path.to_string_lossy().to_string())
        }

        // Video tools
        "video_speed_ramp" => {
            let speed = options.get("speed").and_then(|v| v.as_f64()).unwrap_or(2.0);
            let out_path = parent.join(format!("{}_{:.1}x.{}", stem, speed, ext));
            adjust_video_speed(src, &out_path, speed)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "video_downscale" => {
            let height = options.get("height").and_then(|v| v.as_u64()).unwrap_or(720) as u32;
            let out_path = parent.join(format!("{}_{}p.{}", stem, height, ext));
            downscale_video(src, &out_path, height)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "video_aspect_fit" => {
            let aspect = options.get("aspect").and_then(|v| v.as_str()).unwrap_or("1:1");
            let out_path = parent.join(format!("{}_{}.{}", stem, aspect.replace(':', "x"), ext));
            crop_aspect_ratio(src, &out_path, aspect)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "video_frame_extract" => {
            let frames_dir = parent.join(format!("{}_frames", stem));
            let fps = options.get("fps").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            extract_video_frames(src, &frames_dir, fps)?;
            Ok(frames_dir.to_string_lossy().to_string())
        }

        // Image & Document tools
        "lossless_compress" => {
            let out_path = parent.join(format!("{}_compressed.{}", stem, ext));
            lossless_compress_image(src, &out_path)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "color_space_convert" => {
            let out_path = parent.join(format!("{}_grayscale.{}", stem, ext));
            convert_to_grayscale(src, &out_path)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "favicon_generate" => {
            let out_path = parent.join(format!("{}_favicon.ico", stem));
            generate_favicon_bundle(src, &out_path)?;
            Ok(out_path.to_string_lossy().to_string())
        }
        "pdf_split" => {
            let split_dir = parent.join(format!("{}_pages", stem));
            let pages = split_pdf_pages(src, &split_dir)?;
            Ok(format!("Split into {} pages in {}", pages.len(), split_dir.display()))
        }

        _ => Err(format!("Unrecognized tool kind: {}", tool_id)),
    }
}

pub fn execute_action(file_path: &str, action_id: &str) -> Result<String, String> {
    let src = Path::new(file_path);
    if !src.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    match action_id {
        "reveal_terminal" => {
            reveal_in_terminal(src)?;
            Ok("Terminal spawned successfully".into())
        }
        "copy_path" => Ok(file_path.to_string()),
        "copy_file" => {
            let parent = src.parent().unwrap_or_else(|| Path::new("."));
            let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
            let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");
            let dst = if ext.is_empty() {
                parent.join(format!("{}_copy", stem))
            } else {
                parent.join(format!("{}_copy.{}", stem, ext))
            };
            copy_file(src, &dst)?;
            Ok(dst.to_string_lossy().to_string())
        }
        "move_file" => Ok(format!("Ready to move {}", file_path)),
        _ => Err(format!("Unrecognized action kind: {}", action_id)),
    }
}
