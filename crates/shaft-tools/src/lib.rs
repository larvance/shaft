#![deny(unsafe_code)]

pub mod actions;
pub mod audio;
pub mod dispatcher;
pub mod image_pdf;
pub mod video;

pub use actions::{copy_file, move_file, reveal_in_terminal};
pub use audio::{generate_waveform_video, normalize_loudness, optimize_audio_bitrate, trim_silence};
pub use dispatcher::{execute_action, execute_tool};
pub use image_pdf::{
    convert_to_grayscale, generate_favicon_bundle, lossless_compress_image, split_pdf_pages,
};
pub use video::{adjust_video_speed, crop_aspect_ratio, downscale_video, extract_video_frames};
