use std::path::Path;
use std::process::Command;
use shaft_core::normalize_format;

#[derive(Debug, Clone)]
pub struct VideoTranscodeOptions {
    pub preset: String,
    pub crf: u8,
    pub video_bitrate: Option<String>,
    pub audio_bitrate: Option<String>,
    pub fps: Option<u32>,
    pub scale_width: Option<u32>,
    pub trim_start_sec: Option<f64>,
    pub trim_duration_sec: Option<f64>,
}

impl Default for VideoTranscodeOptions {
    fn default() -> Self {
        Self {
            preset: "fast".to_string(),
            crf: 22,
            video_bitrate: None,
            audio_bitrate: None,
            fps: None,
            scale_width: None,
            trim_start_sec: None,
            trim_duration_sec: None,
        }
    }
}

pub fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn transcode_video<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
    options: &VideoTranscodeOptions,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Source video file does not exist: {}", src.display()));
    }

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let norm_target = normalize_format(target_format);

    // 1. Two-pass GIF synthesis branch
    if norm_target == "gif" {
        return generate_palette_gif(
            src,
            dst,
            options.fps.unwrap_or(15),
            options.scale_width.or(Some(480)),
        );
    }

    // 2. Audio extraction branch (Video -> Audio)
    let is_audio_target = matches!(
        norm_target.as_str(),
        "mp3" | "m4a" | "aac" | "wav" | "flac" | "ogg" | "opus" | "aiff" | "aif" | "wma"
    );

    if is_audio_target {
        return extract_audio_from_video(src, dst, &norm_target, options);
    }

    // 3. Full Video Transcoding branch
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-nostdin");

    if let Some(start) = options.trim_start_sec {
        cmd.arg("-ss").arg(format!("{:.3}", start));
    }

    cmd.arg("-i").arg(src);

    if let Some(dur) = options.trim_duration_sec {
        cmd.arg("-t").arg(format!("{:.3}", dur));
    }

    match norm_target.as_str() {
        "mp4" => {
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg(&options.preset)
                .arg("-crf")
                .arg(options.crf.to_string())
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("192k"));
        }
        "mov" => {
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg(&options.preset)
                .arg("-crf")
                .arg(options.crf.to_string())
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("192k"));
        }
        "mkv" => {
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg(&options.preset)
                .arg("-crf")
                .arg(options.crf.to_string())
                .arg("-c:a")
                .arg("libopus")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("128k"));
        }
        "webm" => {
            cmd.arg("-c:v")
                .arg("libvpx-vp9")
                .arg("-crf")
                .arg("30")
                .arg("-b:v")
                .arg("0")
                .arg("-c:a")
                .arg("libopus")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("128k"));
        }
        "avi" => {
            cmd.arg("-c:v")
                .arg("mpeg4")
                .arg("-q:v")
                .arg("4")
                .arg("-c:a")
                .arg("libmp3lame")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("192k"));
        }
        "wmv" => {
            cmd.arg("-c:v")
                .arg("wmv2")
                .arg("-b:v")
                .arg(options.video_bitrate.as_deref().unwrap_or("2M"))
                .arg("-c:a")
                .arg("wmav2")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("128k"));
        }
        _ => {
            return Err(format!("Unsupported video target format: {}", target_format));
        }
    }

    if let Some(fps) = options.fps {
        cmd.arg("-r").arg(fps.to_string());
    }

    if let Some(width) = options.scale_width {
        cmd.arg("-vf").arg(format!("scale={}:-2", width));
    }

    cmd.arg(dst);

    let output = cmd.output().map_err(|e| {
        format!("Failed to execute ffmpeg command: {}. Ensure ffmpeg is installed.", e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let last_lines: Vec<&str> = stderr.lines().rev().take(6).collect();
        return Err(format!(
            "FFmpeg video transcoding failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            last_lines.into_iter().rev().collect::<Vec<_>>().join("\n")
        ));
    }

    if !dst.exists() || std::fs::metadata(dst).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err(format!("Output video file was not created or is empty: {}", dst.display()));
    }

    Ok(())
}

fn extract_audio_from_video(
    src: &Path,
    dst: &Path,
    norm_target: &str,
    options: &VideoTranscodeOptions,
) -> Result<(), String> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-nostdin").arg("-i").arg(src).arg("-vn");

    match norm_target {
        "mp3" => {
            cmd.arg("-c:a")
                .arg("libmp3lame")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("256k"));
        }
        "m4a" | "aac" => {
            cmd.arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("256k"));
        }
        "wav" => {
            cmd.arg("-c:a").arg("pcm_s16le");
        }
        "flac" => {
            cmd.arg("-c:a").arg("flac");
        }
        "ogg" => {
            cmd.arg("-c:a")
                .arg("libvorbis")
                .arg("-q:a")
                .arg("6");
        }
        "opus" => {
            cmd.arg("-c:a")
                .arg("libopus")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("128k"));
        }
        "aiff" | "aif" => {
            cmd.arg("-c:a").arg("pcm_s16be");
        }
        "wma" => {
            cmd.arg("-c:a")
                .arg("wmav2")
                .arg("-b:a")
                .arg(options.audio_bitrate.as_deref().unwrap_or("192k"));
        }
        _ => {
            return Err(format!("Unsupported audio target format for extraction: {}", norm_target));
        }
    }

    cmd.arg(dst);

    let output = cmd.output().map_err(|e| {
        format!("Failed to execute ffmpeg audio extraction: {}", e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg audio extraction failed: {}", stderr));
    }

    Ok(())
}

pub fn trim_video_lossless<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    start_sec: f64,
    duration_sec: Option<f64>,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Source video file does not exist: {}", src.display()));
    }

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-nostdin")
        .arg("-ss")
        .arg(format!("{:.3}", start_sec))
        .arg("-i")
        .arg(src);

    if let Some(dur) = duration_sec {
        cmd.arg("-t").arg(format!("{:.3}", dur));
    }

    // Zero-reencoding keyframe copy
    cmd.arg("-c").arg("copy").arg(dst);

    let output = cmd.output().map_err(|e| {
        format!("Failed to execute lossless video trim: {}", e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Lossless video trim failed: {}", stderr));
    }

    Ok(())
}

pub fn generate_palette_gif<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    fps: u32,
    scale_width: Option<u32>,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Source video file does not exist: {}", src.display()));
    }

    let filter = if let Some(width) = scale_width {
        format!(
            "fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=floyd_steinberg",
            fps, width
        )
    } else {
        format!(
            "fps={},split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=floyd_steinberg",
            fps
        )
    };

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(src)
        .arg("-vf")
        .arg(&filter)
        .arg(dst);

    let output = cmd.output().map_err(|e| {
        format!("Failed to execute palettegen GIF synthesis: {}", e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("GIF synthesis failed: {}", stderr));
    }

    if !dst.exists() || std::fs::metadata(dst).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err(format!("Output GIF file was not created: {}", dst.display()));
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
            .arg("sine=frequency=1000:duration=0.5")
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
    fn test_ffmpeg_availability() {
        assert!(is_ffmpeg_available(), "FFmpeg should be installed and available in PATH");
    }

    #[test]
    fn test_mp4_to_webm_and_mov_transcoding() {
        if !is_ffmpeg_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let src_mp4 = temp_dir.join("shaft_test_vid.mp4");
        let dst_webm = temp_dir.join("shaft_test_vid.webm");
        let dst_mov = temp_dir.join("shaft_test_vid.mov");

        assert!(generate_test_video(&src_mp4), "Failed to generate test video");

        let opts = VideoTranscodeOptions::default();

        // 1. MP4 -> WebM
        let res_webm = transcode_video(&src_mp4, &dst_webm, "webm", &opts);
        assert!(res_webm.is_ok(), "MP4 to WebM failed: {:?}", res_webm);
        assert!(dst_webm.exists());
        assert!(std::fs::metadata(&dst_webm).unwrap().len() > 0);

        // 2. MP4 -> MOV
        let res_mov = transcode_video(&src_mp4, &dst_mov, "mov", &opts);
        assert!(res_mov.is_ok(), "MP4 to MOV failed: {:?}", res_mov);
        assert!(dst_mov.exists());
        assert!(std::fs::metadata(&dst_mov).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src_mp4);
        let _ = std::fs::remove_file(&dst_webm);
        let _ = std::fs::remove_file(&dst_mov);
    }

    #[test]
    fn test_mp4_to_gif_palettegen() {
        if !is_ffmpeg_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let src_mp4 = temp_dir.join("shaft_test_vid_gif.mp4");
        let dst_gif = temp_dir.join("shaft_test_vid.gif");

        assert!(generate_test_video(&src_mp4));

        let opts = VideoTranscodeOptions {
            fps: Some(10),
            scale_width: Some(120),
            ..Default::default()
        };

        let res_gif = transcode_video(&src_mp4, &dst_gif, "gif", &opts);
        assert!(res_gif.is_ok(), "MP4 to GIF palettegen failed: {:?}", res_gif);
        assert!(dst_gif.exists());
        assert!(std::fs::metadata(&dst_gif).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src_mp4);
        let _ = std::fs::remove_file(&dst_gif);
    }

    #[test]
    fn test_audio_extraction_from_mp4() {
        if !is_ffmpeg_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let src_mp4 = temp_dir.join("shaft_test_vid_extract.mp4");
        let dst_mp3 = temp_dir.join("shaft_test_extract.mp3");

        assert!(generate_test_video(&src_mp4));

        let opts = VideoTranscodeOptions::default();
        let res_mp3 = transcode_video(&src_mp4, &dst_mp3, "mp3", &opts);
        assert!(res_mp3.is_ok(), "Audio extraction to MP3 failed: {:?}", res_mp3);
        assert!(dst_mp3.exists());
        assert!(std::fs::metadata(&dst_mp3).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src_mp4);
        let _ = std::fs::remove_file(&dst_mp3);
    }

    #[test]
    fn test_lossless_video_trimming() {
        if !is_ffmpeg_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let src_mp4 = temp_dir.join("shaft_test_vid_trim.mp4");
        let dst_trimmed = temp_dir.join("shaft_test_vid_trimmed.mp4");

        assert!(generate_test_video(&src_mp4));

        let res_trim = trim_video_lossless(&src_mp4, &dst_trimmed, 0.1, Some(0.3));
        assert!(res_trim.is_ok(), "Lossless trim failed: {:?}", res_trim);
        assert!(dst_trimmed.exists());
        assert!(std::fs::metadata(&dst_trimmed).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src_mp4);
        let _ = std::fs::remove_file(&dst_trimmed);
    }

    #[test]
    fn test_video_non_existent_source() {
        let non_existent = Path::new("/path/to/missing/video.mp4");
        let dst = Path::new("/tmp/should_fail.webm");
        let opts = VideoTranscodeOptions::default();
        let res = transcode_video(non_existent, dst, "webm", &opts);
        assert!(res.is_err());
    }
}
