use std::path::Path;
use shaft_core::types::FileCategory;
use crate::markdown::transcode_document;
use crate::pdf::transcode_pdf;
use crate::spreadsheet::transcode_spreadsheet;
use crate::subtitles::transcode_subtitle;

pub fn transcode_by_category<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
    category: FileCategory,
) -> Result<(), String> {
    match category {
        FileCategory::Spreadsheet => transcode_spreadsheet(source_path, output_path, target_format),
        FileCategory::Pdf => transcode_pdf(source_path, output_path, target_format),
        FileCategory::Subtitle => transcode_subtitle(source_path, output_path, target_format),
        FileCategory::Document | FileCategory::Ebook | FileCategory::Unknown | FileCategory::Image | FileCategory::Audio | FileCategory::Video | FileCategory::Presentation | FileCategory::Archive => {
            transcode_document(source_path, output_path, target_format)
        }
    }
}
