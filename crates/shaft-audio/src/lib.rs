#![deny(unsafe_code)]

pub mod transcode;

pub use transcode::{is_ffmpeg_available, transcode_audio, AudioTranscodeOptions};
