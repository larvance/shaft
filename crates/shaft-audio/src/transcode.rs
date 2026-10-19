use std::path::Path;
use std::process::Command;
use shaft_core::normalize_format;

#[derive(Debug, Clone)]
pub struct AudioTranscodeOptions {
    pub bitrate_kbps: Option<u32>,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u8>,
}

impl Default for AudioTranscodeOptions {
    fn default() -> Self {
        Self {
            bitrate_kbps: None,
            sample_rate_hz: None,
            channels: None,
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

pub fn transcode_audio<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
    options: &AudioTranscodeOptions,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Source audio file does not exist: {}", src.display()));
    }

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let norm_target = normalize_format(target_format);

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(src);

    match norm_target.as_str() {
        "mp3" => {
            cmd.arg("-c:a").arg("libmp3lame");
            let br = options.bitrate_kbps.unwrap_or(320);
            cmd.arg("-b:a").arg(format!("{}k", br));
        }
        "m4a" | "aac" => {
            cmd.arg("-c:a").arg("aac");
            let br = options.bitrate_kbps.unwrap_or(256);
            cmd.arg("-b:a").arg(format!("{}k", br));
        }
        "wav" => {
            cmd.arg("-c:a").arg("pcm_s16le");
        }
        "flac" => {
            cmd.arg("-c:a").arg("flac");
        }
        "ogg" => {
            cmd.arg("-c:a").arg("libvorbis");
            if let Some(br) = options.bitrate_kbps {
                cmd.arg("-b:a").arg(format!("{}k", br));
            } else {
                cmd.arg("-q:a").arg("6");
            }
        }
        "opus" => {
            cmd.arg("-c:a").arg("libopus");
            let br = options.bitrate_kbps.unwrap_or(128);
            cmd.arg("-b:a").arg(format!("{}k", br));
        }
        "aiff" | "aif" => {
            cmd.arg("-c:a").arg("pcm_s16be");
        }
        "wma" => {
            cmd.arg("-c:a").arg("wmav2");
            let br = options.bitrate_kbps.unwrap_or(192);
            cmd.arg("-b:a").arg(format!("{}k", br));
        }
        _ => {
            return Err(format!("Unsupported audio target format: {}", target_format));
        }
    }

    if let Some(sr) = options.sample_rate_hz {
        cmd.arg("-ar").arg(sr.to_string());
    }

    if let Some(ch) = options.channels {
        cmd.arg("-ac").arg(ch.to_string());
    }

    cmd.arg(dst);

    let output = cmd.output().map_err(|e| {
        format!("Failed to execute ffmpeg command: {}. Ensure ffmpeg is installed.", e)
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let last_lines: Vec<&str> = stderr.lines().rev().take(6).collect();
        return Err(format!(
            "FFmpeg audio transcoding failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            last_lines.into_iter().rev().collect::<Vec<_>>().join("\n")
        ));
    }

    if !dst.exists() || std::fs::metadata(dst).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err(format!("Output file was not created or is empty: {}", dst.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_tone(dest: &Path) -> bool {
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=440:duration=0.3")
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
    fn test_wav_to_mp3_and_ogg_transcoding() {
        if !is_ffmpeg_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let src_wav = temp_dir.join("shaft_test_tone.wav");
        let dst_mp3 = temp_dir.join("shaft_test_tone.mp3");
        let dst_ogg = temp_dir.join("shaft_test_tone.ogg");

        assert!(generate_test_tone(&src_wav), "Failed to generate test audio tone");

        let opts = AudioTranscodeOptions::default();

        // 1. WAV -> MP3
        let res_mp3 = transcode_audio(&src_wav, &dst_mp3, "mp3", &opts);
        assert!(res_mp3.is_ok(), "WAV to MP3 failed: {:?}", res_mp3);
        assert!(dst_mp3.exists());
        assert!(std::fs::metadata(&dst_mp3).unwrap().len() > 0);

        // 2. MP3 -> OGG
        let res_ogg = transcode_audio(&dst_mp3, &dst_ogg, "ogg", &opts);
        assert!(res_ogg.is_ok(), "MP3 to OGG failed: {:?}", res_ogg);
        assert!(dst_ogg.exists());
        assert!(std::fs::metadata(&dst_ogg).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src_wav);
        let _ = std::fs::remove_file(&dst_mp3);
        let _ = std::fs::remove_file(&dst_ogg);
    }

    #[test]
    fn test_wav_to_flac_and_opus() {
        if !is_ffmpeg_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let src_wav = temp_dir.join("shaft_test_tone2.wav");
        let dst_flac = temp_dir.join("shaft_test_tone2.flac");
        let dst_opus = temp_dir.join("shaft_test_tone2.opus");

        assert!(generate_test_tone(&src_wav));

        let opts = AudioTranscodeOptions {
            bitrate_kbps: Some(128),
            sample_rate_hz: Some(48000),
            channels: Some(2),
        };

        assert!(transcode_audio(&src_wav, &dst_flac, "flac", &opts).is_ok());
        assert!(transcode_audio(&src_wav, &dst_opus, "opus", &opts).is_ok());

        assert!(dst_flac.exists());
        assert!(dst_opus.exists());

        let _ = std::fs::remove_file(&src_wav);
        let _ = std::fs::remove_file(&dst_flac);
        let _ = std::fs::remove_file(&dst_opus);
    }

    #[test]
    fn test_audio_non_existent_source() {
        let non_existent = Path::new("/path/to/missing/file.wav");
        let dst = Path::new("/tmp/should_fail.mp3");
        let opts = AudioTranscodeOptions::default();
        let res = transcode_audio(non_existent, dst, "mp3", &opts);
        assert!(res.is_err());
    }

    #[test]
    fn test_unsupported_audio_format() {
        let temp_dir = std::env::temp_dir();
        let src_wav = temp_dir.join("shaft_dummy.wav");
        let dst = temp_dir.join("shaft_dummy.xyz");
        assert!(generate_test_tone(&src_wav));

        let opts = AudioTranscodeOptions::default();
        let res = transcode_audio(&src_wav, &dst, "xyz", &opts);
        assert!(res.is_err());

        let _ = std::fs::remove_file(&src_wav);
    }
}
