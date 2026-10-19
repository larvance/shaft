use std::path::Path;
use shaft_core::normalize_format;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleCue {
    pub index: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub fn parse_srt(content: &str) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let lines: Vec<&str> = block.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        if lines.len() < 2 {
            continue;
        }

        let mut line_idx = 0;
        let cue_idx = if let Ok(idx) = lines[0].parse::<usize>() {
            line_idx += 1;
            idx
        } else {
            cues.len() + 1
        };

        if line_idx >= lines.len() {
            continue;
        }

        let time_line = lines[line_idx];
        if let Some((start_ms, end_ms)) = parse_time_range(time_line) {
            line_idx += 1;
            let text = lines[line_idx..].join("\n");
            cues.push(SubtitleCue {
                index: cue_idx,
                start_ms,
                end_ms,
                text,
            });
        }
    }

    cues
}

pub fn parse_vtt(content: &str) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();
    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();
    let mut i = 0;

    // Skip WEBVTT header and metadata
    while i < lines.len() {
        if lines[i].starts_with("WEBVTT") || lines[i].starts_with("NOTE") || lines[i].is_empty() {
            i += 1;
        } else {
            break;
        }
    }

    let mut cue_counter = 1;
    while i < lines.len() {
        if lines[i].is_empty() {
            i += 1;
            continue;
        }

        // Could be cue identifier or timecode line directly
        let time_line = if lines[i].contains("-->") {
            let l = lines[i];
            i += 1;
            l
        } else if i + 1 < lines.len() && lines[i + 1].contains("-->") {
            i += 1;
            let l = lines[i];
            i += 1;
            l
        } else {
            i += 1;
            continue;
        };

        if let Some((start_ms, end_ms)) = parse_time_range(time_line) {
            let mut text_lines = Vec::new();
            while i < lines.len() && !lines[i].is_empty() {
                text_lines.push(lines[i]);
                i += 1;
            }

            cues.push(SubtitleCue {
                index: cue_counter,
                start_ms,
                end_ms,
                text: text_lines.join("\n"),
            });
            cue_counter += 1;
        }
    }

    cues
}

fn parse_time_range(line: &str) -> Option<(u64, u64)> {
    let parts: Vec<&str> = line.split("-->").collect();
    if parts.len() != 2 {
        return None;
    }

    let start_str = parts[0].trim().split_whitespace().next()?;
    let end_str = parts[1].trim().split_whitespace().next()?;

    let start_ms = parse_timestamp(start_str)?;
    let end_ms = parse_timestamp(end_str)?;

    Some((start_ms, end_ms))
}

fn parse_timestamp(ts: &str) -> Option<u64> {
    // Reconcile comma or dot separator for milliseconds
    let sanitized = ts.replace(',', ".");
    let parts: Vec<&str> = sanitized.split(':').collect();

    match parts.len() {
        3 => {
            let hours = parts[0].parse::<u64>().ok()?;
            let minutes = parts[1].parse::<u64>().ok()?;
            let sec_parts: Vec<&str> = parts[2].split('.').collect();
            let seconds = sec_parts[0].parse::<u64>().ok()?;
            let millis = if sec_parts.len() > 1 {
                let ms_str = format!("{:0<3}", sec_parts[1]);
                ms_str[..3.min(ms_str.len())].parse::<u64>().ok()?
            } else {
                0
            };
            Some(((hours * 3600 + minutes * 60 + seconds) * 1000) + millis)
        }
        2 => {
            let minutes = parts[0].parse::<u64>().ok()?;
            let sec_parts: Vec<&str> = parts[1].split('.').collect();
            let seconds = sec_parts[0].parse::<u64>().ok()?;
            let millis = if sec_parts.len() > 1 {
                let ms_str = format!("{:0<3}", sec_parts[1]);
                ms_str[..3.min(ms_str.len())].parse::<u64>().ok()?
            } else {
                0
            };
            Some(((minutes * 60 + seconds) * 1000) + millis)
        }
        _ => None,
    }
}

pub fn format_srt_time(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let rem = ms % 3_600_000;
    let minutes = rem / 60_000;
    let rem2 = rem % 60_000;
    let seconds = rem2 / 1_000;
    let millis = rem2 % 1_000;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
}

pub fn format_vtt_time(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let rem = ms % 3_600_000;
    let minutes = rem / 60_000;
    let rem2 = rem % 60_000;
    let seconds = rem2 / 1_000;
    let millis = rem2 % 1_000;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

pub fn serialize_srt(cues: &[SubtitleCue]) -> String {
    let mut out = String::new();
    for (i, cue) in cues.iter().enumerate() {
        out.push_str(&format!("{}\n", i + 1));
        out.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(cue.start_ms),
            format_srt_time(cue.end_ms)
        ));
        out.push_str(&cue.text);
        out.push_str("\n\n");
    }
    out
}

pub fn serialize_vtt(cues: &[SubtitleCue]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for (i, cue) in cues.iter().enumerate() {
        out.push_str(&format!("{}\n", i + 1));
        out.push_str(&format!(
            "{} --> {}\n",
            format_vtt_time(cue.start_ms),
            format_vtt_time(cue.end_ms)
        ));
        out.push_str(&cue.text);
        out.push_str("\n\n");
    }
    out
}

pub fn serialize_txt(cues: &[SubtitleCue]) -> String {
    let mut out = String::new();
    for cue in cues {
        out.push_str(&cue.text);
        out.push('\n');
    }
    out
}

pub fn transcode_subtitle<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Subtitle file does not exist: {}", src.display()));
    }

    let content = std::fs::read_to_string(src)
        .map_err(|e| format!("Failed to read subtitle file: {}", e))?;

    let ext = src
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    let cues = if ext == "vtt" || content.starts_with("WEBVTT") {
        parse_vtt(&content)
    } else {
        parse_srt(&content)
    };

    let norm_target = normalize_format(target_format);
    let output_str = match norm_target.as_str() {
        "vtt" => serialize_vtt(&cues),
        "srt" => serialize_srt(&cues),
        "txt" => serialize_txt(&cues),
        _ => return Err(format!("Unsupported subtitle target format: {}", target_format)),
    };

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    std::fs::write(dst, output_str).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_to_vtt_conversion() {
        let srt_content = "1\n00:00:01,500 --> 00:00:04,000\nAutonomous drone system active.\n\n2\n00:00:04,500 --> 00:00:08,250\nTelemetry lock acquired at 8kHz.\n";

        let cues = parse_srt(srt_content);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start_ms, 1500);
        assert_eq!(cues[0].end_ms, 4000);
        assert_eq!(cues[0].text, "Autonomous drone system active.");
        assert_eq!(cues[1].start_ms, 4500);
        assert_eq!(cues[1].end_ms, 8250);

        let vtt = serialize_vtt(&cues);
        assert!(vtt.starts_with("WEBVTT"));
        assert!(vtt.contains("00:00:01.500 --> 00:00:04.000"));
        assert!(vtt.contains("00:00:04.500 --> 00:00:08.250"));

        let reparsed_vtt_cues = parse_vtt(&vtt);
        assert_eq!(reparsed_vtt_cues.len(), 2);
        assert_eq!(reparsed_vtt_cues[0].start_ms, 1500);
        assert_eq!(reparsed_vtt_cues[1].text, "Telemetry lock acquired at 8kHz.");
    }

    #[test]
    fn test_subtitle_to_txt() {
        let srt_content = "1\n00:00:01,000 --> 00:00:02,000\nHello World\n\n2\n00:00:03,000 --> 00:00:04,000\nSystems Online\n";
        let cues = parse_srt(srt_content);
        let txt = serialize_txt(&cues);
        assert_eq!(txt, "Hello World\nSystems Online\n");
    }
}
