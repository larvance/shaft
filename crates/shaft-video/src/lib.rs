#![deny(unsafe_code)]

pub mod transcode;

pub use transcode::{
    generate_palette_gif, is_ffmpeg_available, transcode_video, trim_video_lossless,
    VideoTranscodeOptions,
};
