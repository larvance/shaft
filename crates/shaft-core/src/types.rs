use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    Image,
    Audio,
    Video,
    Pdf,
    Presentation,
    Spreadsheet,
    Document,
    Ebook,
    Subtitle,
    Archive,
    Unknown,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Pdf => "pdf",
            Self::Presentation => "presentation",
            Self::Spreadsheet => "spreadsheet",
            Self::Document => "document",
            Self::Ebook => "ebook",
            Self::Subtitle => "subtitle",
            Self::Archive => "archive",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    LosslessCompress,
    TargetSizeCompress,
    VideoDownscale,
    AudioBitrateOptimize,
    MetadataScrub,
    AutoFaceBlur,
    TextRedact,
    OcrTextExtract,
    ColorSpaceConvert,
    AlphaRemove,
    FaviconGenerate,
    AudioSilenceTrim,
    LoudnessNormalize,
    ProfanityBleep,
    WaveformVideo,
    StemIsolate,
    VideoLosslessTrim,
    VideoSpeedRamp,
    SmoothGif,
    VideoFrameExtract,
    VideoAspectFit,
    PdfSplit,
    PdfMerge,
    TableToCsv,
    EbookCoverExtract,
    ArchiveInspect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    QuickShare,
    FloatQuickLook,
    CopyFile,
    MoveFile,
    CopyPath,
    RevealTerminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub id: ToolKind,
    pub label: String,
    pub description: String,
    pub category: FileCategory,
    pub icon_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub id: ActionKind,
    pub label: String,
    pub description: String,
    pub icon_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtendedMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_seconds: Option<f32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub page_count: Option<u32>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProbeResult {
    pub file_path: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub detected_format: String,
    pub category: FileCategory,
    pub available_targets: Vec<String>,
    pub applicable_tools: Vec<ToolInfo>,
    pub metadata: ExtendedMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionTask {
    pub id: String,
    pub source_path: String,
    pub source_name: String,
    pub source_format: String,
    pub target_format: String,
    pub category: FileCategory,
    pub status: TaskStatus,
    pub progress: f32,
    pub error_message: Option<String>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressEvent {
    pub task_id: String,
    pub status: TaskStatus,
    pub progress: f32,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: Option<u64>,
    pub eta_seconds: Option<f32>,
    pub error_message: Option<String>,
    pub output_path: Option<String>,
}
