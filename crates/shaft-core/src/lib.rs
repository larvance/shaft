#![deny(unsafe_code)]

pub mod dag;
pub mod probe;
pub mod queue;
pub mod registry;
pub mod types;

pub use dag::{ConversionDAG, ConversionRoute};
pub use probe::probe_file;
pub use queue::{CancellationToken, TaskQueueManager};
pub use registry::{
    get_all_conversions, get_applicable_tools, get_available_targets, get_category_for_format,
    get_file_actions, normalize_format,
};
pub use types::{
    ActionInfo, ActionKind, ConversionTask, ExtendedMetadata, FileCategory, FileProbeResult,
    TaskProgressEvent, TaskStatus, ToolInfo, ToolKind,
};
