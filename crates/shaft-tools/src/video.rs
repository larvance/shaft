use std::path::Path;
use std::process::Command;

pub fn adjust_video_speed<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    speed: f64,
) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source video does not exist: {}", s.display()));
    }

    if speed <= 0.05 || speed > 10.0 {
        return Err("Speed factor must be between 0.1x and 10.0x".into());
    }

    let video_pts = format!("setpts={:.4}*PTS", 1.0 / speed);

    // atempo filter is bounded between 0.5 and 2.0, chain if necessary
    let audio_tempo = if (0.5..=2.0).contains(&speed) {
        format!("atempo={:.3}", speed)
    } else if speed > 2.0 {
        let half = speed / 2.0;
        format!("atempo=2.0,atempo={:.3}", half.min(2.0))
    } else {
        let double = speed * 2.0;
        format!("atempo=0.5,atempo={:.3}", double.max(0.5))
    };

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-filter:v")
        .arg(&video_pts)
        .arg("-filter:a")
        .arg(&audio_tempo)
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to adjust video speed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Video speed ramp failed: {}", stderr));
    }

    Ok(())
}

pub fn downscale_video<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    target_height: u32,
) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source video does not exist: {}", s.display()));
    }

    let filter = format!("scale=-2:{}", target_height);

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-vf")
        .arg(&filter)
        .arg("-c:a")
        .arg("copy")
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to downscale video: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Video downscaling failed: {}", stderr));
    }

    Ok(())
}

pub fn crop_aspect_ratio<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    aspect_ratio: &str,
) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source video does not exist: {}", s.display()));
    }

    let filter = match aspect_ratio {
        "1:1" => "crop=min(iw\\,ih):min(iw\\,ih)",
        "9:16" => "crop=ih*9/16:ih",
        "16:9" => "crop=iw:iw*9/16",
        "4:5" => "crop=ih*4/5:ih",
        _ => return Err(format!("Unsupported aspect ratio target: {}", aspect_ratio)),
    };

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-vf")
        .arg(filter)
        .arg("-c:a")
        .arg("copy")
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to crop video aspect ratio: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Video crop failed: {}", stderr));
    }

    Ok(())
}

pub fn extract_video_frames<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    output_dir: Q,
    fps: f32,
) -> Result<(), String> {
    let s = src.as_ref();
    let d = output_dir.as_ref();

    if !s.exists() {
        return Err(format!("Source video does not exist: {}", s.display()));
    }

    if !d.exists() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }

    let out_pattern = d.join("frame_%04d.png");

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-vf")
        .arg(format!("fps={:.2}", fps))
        .arg(out_pattern)
        .output()
        .map_err(|e| format!("Failed to extract frames: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Frame extraction failed: {}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_video(dest: &Path) -> bool {
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("testsrc=duration=0.5:size=160x120:rate=15")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=440:duration=0.5")
            .arg("-c:v")
            .arg("libx264")
            .arg("-c:a")
            .arg("aac")
            .arg(dest)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_video_speed_ramp() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_speed_in.mp4");
        let dst = temp.join("shaft_speed_out.mp4");

        assert!(generate_test_video(&src));

        let res = adjust_video_speed(&src, &dst, 2.0);
        assert!(res.is_ok(), "Video speed ramp failed: {:?}", res);
        assert!(dst.exists());
        assert!(std::fs::metadata(&dst).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    }

    #[test]
    fn test_video_crop_aspect() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_crop_in.mp4");
        let dst = temp.join("shaft_crop_out.mp4");

        assert!(generate_test_video(&src));

        let res = crop_aspect_ratio(&src, &dst, "1:1");
        assert!(res.is_ok(), "Video aspect crop failed: {:?}", res);
        assert!(dst.exists());

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    }
}
