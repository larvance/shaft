#![deny(unsafe_code)]

pub mod markdown;
pub mod pdf;
pub mod spreadsheet;
pub mod subtitles;
pub mod transcode;

pub use markdown::{markdown_to_html, markdown_to_plaintext, transcode_document};
pub use pdf::{extract_text_from_pdf, transcode_pdf};
pub use spreadsheet::{read_spreadsheet, transcode_spreadsheet, write_spreadsheet};
pub use subtitles::{
    format_srt_time, format_vtt_time, parse_srt, parse_vtt, serialize_srt, serialize_txt,
    serialize_vtt, transcode_subtitle, SubtitleCue,
};
pub use transcode::transcode_by_category;
