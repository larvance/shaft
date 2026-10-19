#![deny(unsafe_code)]

pub mod probe;
pub mod registry;
pub mod types;

pub use probe::probe_file;
pub use registry::{
    get_all_conversions, get_applicable_tools, get_available_targets, get_category_for_format,
    get_file_actions, normalize_format,
};
pub use types::{
    ActionInfo, ActionKind, ConversionTask, FileCategory, FileProbeResult, TaskStatus, ToolInfo,
    ToolKind,
};
