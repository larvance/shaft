use std::path::Path;
use std::process::Command;

pub fn trim_silence<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source audio file does not exist: {}", s.display()));
    }

    let filter = "silenceremove=start_periods=1:start_duration=0.2:start_threshold=-50dB:stop_periods=1:stop_duration=0.5:stop_threshold=-50dB";

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-af")
        .arg(filter)
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to execute silence trim: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Silence trim failed: {}", stderr));
    }

    Ok(())
}

pub fn normalize_loudness<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source audio file does not exist: {}", s.display()));
    }

    // Standard EBU R128 (-14 LUFS integrated loudness, -1.5 dB true peak)
    let filter = "loudnorm=I=-14:LRA=11:TP=-1.5";

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-af")
        .arg(filter)
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to execute loudness normalization: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Loudness normalization failed: {}", stderr));
    }

    Ok(())
}

pub fn generate_waveform_video<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source audio file does not exist: {}", s.display()));
    }

    let filter = "[0:a]showwaves=s=1280x720:mode=line:colors=0x5e6ad2[v]";

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-filter_complex")
        .arg(filter)
        .arg("-map")
        .arg("[v]")
        .arg("-map")
        .arg("0:a")
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-c:a")
        .arg("aac")
        .arg("-b:a")
        .arg("192k")
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to execute waveform video generator: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Waveform video generation failed: {}", stderr));
    }

    Ok(())
}

pub fn optimize_audio_bitrate<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    bitrate_kbps: u32,
) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source audio file does not exist: {}", s.display()));
    }

    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-nostdin")
        .arg("-i")
        .arg(s)
        .arg("-b:a")
        .arg(format!("{}k", bitrate_kbps))
        .arg(d)
        .output()
        .map_err(|e| format!("Failed to optimize audio bitrate: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Audio bitrate optimization failed: {}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sine_wave(dest: &Path) -> bool {
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg("sine=frequency=440:duration=0.5")
            .arg(dest)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_normalize_loudness() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_norm_in.wav");
        let dst = temp.join("shaft_norm_out.wav");

        assert!(generate_sine_wave(&src));

        let res = normalize_loudness(&src, &dst);
        assert!(res.is_ok(), "Loudness normalization failed: {:?}", res);
        assert!(dst.exists());
        assert!(std::fs::metadata(&dst).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    }

    #[test]
    fn test_waveform_video_generation() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_wave_in.wav");
        let dst = temp.join("shaft_wave_out.mp4");

        assert!(generate_sine_wave(&src));

        let res = generate_waveform_video(&src, &dst);
        assert!(res.is_ok(), "Waveform video generation failed: {:?}", res);
        assert!(dst.exists());
        assert!(std::fs::metadata(&dst).unwrap().len() > 0);

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
    }
}
