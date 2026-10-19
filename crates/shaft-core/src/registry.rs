use std::collections::HashMap;
use crate::types::{ActionInfo, ActionKind, FileCategory, ToolInfo, ToolKind};

pub fn get_category_for_format(ext: &str) -> FileCategory {
    let lower = ext.trim_start_matches('.').to_ascii_lowercase();
    match lower.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "tiff" | "tif" | "svg" | "avif" | "bmp" => {
            FileCategory::Image
        }
        "mp3" | "m4a" | "wav" | "flac" | "ogg" | "opus" | "aiff" | "aif" | "wma" | "aac" => {
            FileCategory::Audio
        }
        "mp4" | "mov" | "mkv" | "webm" | "avi" | "wmv" | "gif" => {
            FileCategory::Video
        }
        "pdf" => FileCategory::Pdf,
        "pptx" | "ppt" => FileCategory::Presentation,
        "xlsx" | "xls" | "ods" | "csv" | "tsv" => FileCategory::Spreadsheet,
        "docx" | "pages" | "md" | "markdown" | "rtf" | "rtfd" | "textclipping" | "txt" => {
            FileCategory::Document
        }
        "epub" => FileCategory::Ebook,
        "srt" | "vtt" => FileCategory::Subtitle,
        "zip" | "tar" | "gz" | "tgz" | "rar" => FileCategory::Archive,
        _ => FileCategory::Unknown,
    }
}

pub fn get_available_targets(source_format: &str) -> Vec<&'static str> {
    let norm = normalize_format(source_format);
    match norm.as_str() {
        // Category 1: Images
        "jpg" => vec!["png", "webp", "heic", "tiff", "avif", "bmp", "pdf"],
        "png" => vec!["jpg", "webp", "heic", "tiff", "avif", "bmp", "pdf"],
        "webp" => vec!["jpg", "png", "heic", "tiff", "avif", "bmp", "pdf"],
        "heic" => vec!["jpg", "png", "webp", "tiff", "avif", "bmp", "pdf"],
        "tiff" => vec!["jpg", "png", "webp", "heic", "avif", "bmp", "pdf"],
        "svg" => vec!["jpg", "png", "webp", "heic", "tiff", "avif", "bmp", "pdf"],
        "avif" => vec!["jpg", "png", "webp", "heic", "tiff", "bmp", "pdf"],
        "bmp" => vec!["jpg", "png", "webp", "heic", "tiff", "avif", "pdf"],

        // Category 2: Audio
        "mp3" => vec!["m4a", "wav", "flac", "ogg", "opus", "aiff", "wma"],
        "m4a" => vec!["mp3", "wav", "flac", "ogg", "opus", "aiff", "wma"],
        "wav" => vec!["mp3", "m4a", "flac", "ogg", "opus", "aiff", "wma"],
        "flac" => vec!["mp3", "m4a", "wav", "ogg", "opus", "aiff", "wma"],
        "ogg" => vec!["mp3", "m4a", "wav", "flac", "opus", "aiff", "wma"],
        "opus" => vec!["mp3", "m4a", "wav", "flac", "ogg", "aiff", "wma"],
        "aiff" => vec!["mp3", "m4a", "wav", "flac", "ogg", "opus", "wma"],
        "wma" => vec!["mp3", "m4a", "wav", "flac", "ogg", "opus", "aiff"],

        // Category 3: Video
        "mp4" => vec!["mov", "mkv", "webm", "avi", "wmv", "gif", "mp3"],
        "mov" => vec!["mp4", "mkv", "webm", "avi", "wmv", "gif", "mp3"],
        "mkv" => vec!["mp4", "mov", "webm", "avi", "wmv", "gif", "mp3"],
        "webm" => vec!["mp4", "mov", "mkv", "avi", "wmv", "gif", "mp3"],
        "avi" => vec!["mp4", "mov", "mkv", "webm", "wmv", "gif", "mp3"],
        "wmv" => vec!["mp4", "mov", "mkv", "webm", "avi", "gif", "mp3"],
        "gif" => vec!["mp4", "mov", "mkv", "webm", "avi", "wmv"],

        // Category 4: PDF
        "pdf" => vec!["docx", "pages", "pptx", "xlsx", "jpg", "png", "epub", "txt", "md"],

        // Category 5: Presentations
        "pptx" => vec!["pdf", "jpg", "png"],
        "ppt" => vec!["pdf", "jpg", "png"],

        // Category 6: Spreadsheets
        "xlsx" => vec!["pdf", "ods", "csv", "tsv"],
        "xls" => vec!["pdf", "xlsx", "ods", "csv", "tsv"],
        "ods" => vec!["pdf", "xlsx", "csv", "tsv"],
        "csv" => vec!["pdf", "xlsx", "ods", "tsv"],
        "tsv" => vec!["pdf", "xlsx", "ods", "csv"],

        // Category 7: Documents
        "docx" => vec!["pages", "pdf", "txt", "md", "html", "epub", "png", "jpg"],
        "pages" => vec!["docx", "pdf", "txt", "md", "html", "epub", "png", "jpg"],
        "md" => vec!["docx", "pages", "pdf", "txt", "html", "epub", "png", "jpg"],
        "rtf" => vec!["docx", "pages", "pdf", "txt", "md", "html", "epub", "png", "jpg"],
        "rtfd" => vec!["docx", "pages", "pdf", "txt", "md", "html", "epub", "png", "jpg"],
        "textclipping" => vec!["docx", "pages", "pdf", "txt", "md", "html", "epub", "png", "jpg"],
        "txt" => vec!["docx", "pages", "pdf", "md", "html", "epub", "png", "jpg"],

        // Category 8: E-Books
        "epub" => vec!["pdf", "docx", "pages", "txt", "md", "html"],

        // Category 9: Subtitles
        "srt" => vec!["vtt", "txt"],
        "vtt" => vec!["srt", "txt"],

        // Category 10: Archives
        "zip" => vec!["tar", "gz"],
        "tar" => vec!["zip", "gz"],
        "gz" => vec!["zip", "tar"],
        "rar" => vec!["zip", "tar", "gz"],

        _ => vec![],
    }
}

pub fn normalize_format(input: &str) -> String {
    let s = input.trim_start_matches('.').to_ascii_lowercase();
    match s.as_str() {
        "jpeg" => "jpg".to_string(),
        "tif" => "tiff".to_string(),
        "heif" => "heic".to_string(),
        "markdown" => "md".to_string(),
        "aif" => "aiff".to_string(),
        "tgz" => "gz".to_string(),
        _ => s,
    }
}

pub fn get_applicable_tools(category: FileCategory) -> Vec<ToolInfo> {
    match category {
        FileCategory::Image => vec![
            ToolInfo {
                id: ToolKind::LosslessCompress,
                label: "Lossless Compress".into(),
                description: "Strip metadata and optimize image compression passes".into(),
                category: FileCategory::Image,
                icon_name: "Minimize2".into(),
            },
            ToolInfo {
                id: ToolKind::TargetSizeCompress,
                label: "Target Size".into(),
                description: "Compress image to fit exact file size constraint (e.g. 5 MB)".into(),
                category: FileCategory::Image,
                icon_name: "Sliders".into(),
            },
            ToolInfo {
                id: ToolKind::MetadataScrub,
                label: "Scrub Metadata".into(),
                description: "Remove EXIF, GPS, camera model, and profile metadata".into(),
                category: FileCategory::Image,
                icon_name: "ShieldCheck".into(),
            },
            ToolInfo {
                id: ToolKind::AutoFaceBlur,
                label: "Auto Face Blur".into(),
                description: "Detect faces and apply privacy Gaussian blur".into(),
                category: FileCategory::Image,
                icon_name: "EyeOff".into(),
            },
            ToolInfo {
                id: ToolKind::TextRedact,
                label: "Redact Sensitive Text".into(),
                description: "Blackout personal identifiers, emails, and numbers".into(),
                category: FileCategory::Image,
                icon_name: "FileLock2".into(),
            },
            ToolInfo {
                id: ToolKind::OcrTextExtract,
                label: "Extract Text (OCR)".into(),
                description: "Extract text from image into searchable text".into(),
                category: FileCategory::Image,
                icon_name: "ScanText".into(),
            },
            ToolInfo {
                id: ToolKind::FaviconGenerate,
                label: "Generate Favicon".into(),
                description: "Export multi-layer .ico and Apple Touch icon bundle".into(),
                category: FileCategory::Image,
                icon_name: "Sparkles".into(),
            },
            ToolInfo {
                id: ToolKind::ColorSpaceConvert,
                label: "Convert Color Space".into(),
                description: "Switch between sRGB, Display P3, and Grayscale".into(),
                category: FileCategory::Image,
                icon_name: "Palette".into(),
            },
        ],
        FileCategory::Audio => vec![
            ToolInfo {
                id: ToolKind::AudioSilenceTrim,
                label: "Trim Silence".into(),
                description: "Remove leading and trailing dead silence".into(),
                category: FileCategory::Audio,
                icon_name: "Scissors".into(),
            },
            ToolInfo {
                id: ToolKind::LoudnessNormalize,
                label: "Normalize Loudness".into(),
                description: "Apply standard -14 LUFS loudness curve".into(),
                category: FileCategory::Audio,
                icon_name: "Volume2".into(),
            },
            ToolInfo {
                id: ToolKind::AudioBitrateOptimize,
                label: "Optimize Bitrate".into(),
                description: "Compress audio to efficient 128k/192k bitrate".into(),
                category: FileCategory::Audio,
                icon_name: "FileAudio".into(),
            },
            ToolInfo {
                id: ToolKind::ProfanityBleep,
                label: "Bleep Filter".into(),
                description: "Overlay 1kHz tone over sensitive segments".into(),
                category: FileCategory::Audio,
                icon_name: "MicOff".into(),
            },
            ToolInfo {
                id: ToolKind::WaveformVideo,
                label: "Generate Waveform Video".into(),
                description: "Create fluid obsidian audio visualizer MP4".into(),
                category: FileCategory::Audio,
                icon_name: "Activity".into(),
            },
        ],
        FileCategory::Video => vec![
            ToolInfo {
                id: ToolKind::VideoLosslessTrim,
                label: "Lossless Trim".into(),
                description: "Cut video cleanly on keyframes without re-encoding".into(),
                category: FileCategory::Video,
                icon_name: "Scissors".into(),
            },
            ToolInfo {
                id: ToolKind::SmoothGif,
                label: "Convert to Smooth GIF".into(),
                description: "Optimal 256-color palettegen high-framerate GIF".into(),
                category: FileCategory::Video,
                icon_name: "Film".into(),
            },
            ToolInfo {
                id: ToolKind::VideoDownscale,
                label: "Downscale Resolution".into(),
                description: "Reduce resolution to 1080p or 720p".into(),
                category: FileCategory::Video,
                icon_name: "Tv".into(),
            },
            ToolInfo {
                id: ToolKind::VideoSpeedRamp,
                label: "Speed Ramp".into(),
                description: "Adjust playback rate between 0.25x and 4x".into(),
                category: FileCategory::Video,
                icon_name: "FastForward".into(),
            },
            ToolInfo {
                id: ToolKind::VideoFrameExtract,
                label: "Extract Frames".into(),
                description: "Dump full-resolution keyframe images".into(),
                category: FileCategory::Video,
                icon_name: "Layers".into(),
            },
            ToolInfo {
                id: ToolKind::VideoAspectFit,
                label: "Crop Aspect Ratio".into(),
                description: "Transform video to 16:9, 9:16 vertical, or 1:1".into(),
                category: FileCategory::Video,
                icon_name: "Crop".into(),
            },
        ],
        FileCategory::Pdf | FileCategory::Document => vec![
            ToolInfo {
                id: ToolKind::PdfSplit,
                label: "Split Pages".into(),
                description: "Extract specific page ranges into new documents".into(),
                category: FileCategory::Pdf,
                icon_name: "Split".into(),
            },
            ToolInfo {
                id: ToolKind::PdfMerge,
                label: "Merge Documents".into(),
                description: "Combine multiple PDF documents into one".into(),
                category: FileCategory::Pdf,
                icon_name: "GitMerge".into(),
            },
            ToolInfo {
                id: ToolKind::OcrTextExtract,
                label: "Extract Text".into(),
                description: "Run OCR on document scans".into(),
                category: FileCategory::Pdf,
                icon_name: "ScanText".into(),
            },
            ToolInfo {
                id: ToolKind::LosslessCompress,
                label: "Compress PDF".into(),
                description: "Optimize PDF structure and embedded streams".into(),
                category: FileCategory::Pdf,
                icon_name: "FileDown".into(),
            },
            ToolInfo {
                id: ToolKind::TableToCsv,
                label: "Extract Tables to CSV".into(),
                description: "Detect tabular grid structures and export CSV".into(),
                category: FileCategory::Document,
                icon_name: "Table".into(),
            },
        ],
        FileCategory::Spreadsheet => vec![
            ToolInfo {
                id: ToolKind::TableToCsv,
                label: "Extract to CSV/TSV".into(),
                description: "Clean tabular export".into(),
                category: FileCategory::Spreadsheet,
                icon_name: "Table".into(),
            },
        ],
        FileCategory::Ebook => vec![
            ToolInfo {
                id: ToolKind::EbookCoverExtract,
                label: "Extract Cover Art".into(),
                description: "Extract high-resolution book cover image".into(),
                category: FileCategory::Ebook,
                icon_name: "Image".into(),
            },
        ],
        FileCategory::Archive => vec![
            ToolInfo {
                id: ToolKind::ArchiveInspect,
                label: "Inspect Archive".into(),
                description: "Inspect archive directory structure in RAM".into(),
                category: FileCategory::Archive,
                icon_name: "FolderSearch".into(),
            },
        ],
        _ => vec![],
    }
}

pub fn get_file_actions() -> Vec<ActionInfo> {
    vec![
        ActionInfo {
            id: ActionKind::QuickShare,
            label: "Quick Share".into(),
            description: "Share via local network, AirDrop, or QR server".into(),
            icon_name: "Share2".into(),
        },
        ActionInfo {
            id: ActionKind::FloatQuickLook,
            label: "Float QuickLook".into(),
            description: "Pin floating borderless preview window on top".into(),
            icon_name: "Maximize2".into(),
        },
        ActionInfo {
            id: ActionKind::CopyFile,
            label: "Copy File".into(),
            description: "Copy file payload to system clipboard".into(),
            icon_name: "Copy".into(),
        },
        ActionInfo {
            id: ActionKind::MoveFile,
            label: "Move File".into(),
            description: "Move file to chosen destination folder".into(),
            icon_name: "FolderInput".into(),
        },
        ActionInfo {
            id: ActionKind::CopyPath,
            label: "Copy Absolute Path".into(),
            description: "Copy path string to clipboard".into(),
            icon_name: "Link".into(),
        },
        ActionInfo {
            id: ActionKind::RevealTerminal,
            label: "Reveal in Terminal".into(),
            description: "Open working shell in file directory".into(),
            icon_name: "Terminal".into(),
        },
    ]
}

pub fn get_all_conversions() -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    let formats = vec![
        "jpg", "png", "webp", "heic", "tiff", "svg", "avif", "bmp",
        "mp3", "m4a", "wav", "flac", "ogg", "opus", "aiff", "wma",
        "mp4", "mov", "mkv", "webm", "avi", "wmv", "gif",
        "pdf", "pptx", "ppt", "xlsx", "xls", "ods", "csv", "tsv",
        "docx", "pages", "md", "rtf", "rtfd", "textclipping", "txt",
        "epub", "srt", "vtt", "zip", "tar", "gz", "rar"
    ];

    for f in formats {
        let targets = get_available_targets(f);
        if !targets.is_empty() {
            map.insert(
                f.to_string(),
                targets.into_iter().map(|s| s.to_string()).collect(),
            );
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_conversion_paths() {
        let conversions = get_all_conversions();
        let total: usize = conversions.values().map(|v| v.len()).sum();
        // 272 defined paths + aliases
        assert!(total >= 272, "Total conversion paths must be at least 272, got {}", total);
    }

    #[test]
    fn test_audio_bidirectional() {
        let audio_formats = ["mp3", "m4a", "wav", "flac", "ogg", "opus", "aiff", "wma"];
        for src in audio_formats {
            let targets = get_available_targets(src);
            assert_eq!(targets.len(), 7, "Each audio codec must convert to 7 others");
        }
    }
}
