// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use shaft_core::{
    get_all_conversions, get_applicable_tools, get_available_targets,
    get_file_actions, probe_file, ActionInfo, ConversionTask, FileCategory,
    FileProbeResult, TaskStatus, ToolInfo,
};
use tauri::{Manager, WebviewWindow};

#[tauri::command]
fn probe_path(path: String) -> Result<FileProbeResult, String> {
    probe_file(path)
}

#[tauri::command]
fn get_matrix() -> HashMap<String, Vec<String>> {
    get_all_conversions()
}

#[tauri::command]
fn get_tools_for_category(category: String) -> Vec<ToolInfo> {
    let cat = match category.to_ascii_lowercase().as_str() {
        "image" => FileCategory::Image,
        "audio" => FileCategory::Audio,
        "video" => FileCategory::Video,
        "pdf" => FileCategory::Pdf,
        "presentation" => FileCategory::Presentation,
        "spreadsheet" => FileCategory::Spreadsheet,
        "document" => FileCategory::Document,
        "ebook" => FileCategory::Ebook,
        "subtitle" => FileCategory::Subtitle,
        "archive" => FileCategory::Archive,
        _ => FileCategory::Unknown,
    };
    get_applicable_tools(cat)
}

#[tauri::command]
fn get_actions() -> Vec<ActionInfo> {
    get_file_actions()
}

#[tauri::command]
fn get_targets(format: String) -> Vec<String> {
    get_available_targets(&format)
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

#[tauri::command]
fn pick_files() -> Result<Vec<String>, String> {
    let dialog = rfd::FileDialog::new()
        .set_title("Select Files to Convert with Shaft");

    let Some(paths) = dialog.pick_files() else {
        return Ok(vec![]);
    };

    Ok(paths
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
fn execute_conversion(mut task: ConversionTask) -> Result<ConversionTask, String> {
    // In Phase 1 foundation, mock transition from Queued -> Completed
    let src = Path::new(&task.source_path);
    if !src.exists() {
        task.status = TaskStatus::Failed;
        task.error_message = Some("Source file does not exist".into());
        return Ok(task);
    }

    let parent = src.parent().unwrap_or_else(|| Path::new("."));
    let stem = src
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let out_name = format!("{}_converted.{}", stem, task.target_format);
    let out_path = parent.join(out_name);

    // Copy or write output stub for phase 1 verification
    if let Err(e) = std::fs::copy(src, &out_path) {
        task.status = TaskStatus::Failed;
        task.error_message = Some(format!("Transcoding IO error: {}", e));
    } else {
        task.status = TaskStatus::Completed;
        task.progress = 1.0;
        task.output_path = Some(out_path.to_string_lossy().to_string());
    }

    Ok(task)
}

#[tauri::command]
fn minimize_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_maximize_window(window: WebviewWindow) -> Result<(), String> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn close_window(window: WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
fn start_window_dragging(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

fn apply_window_vibrancy(_window: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
        let _ = apply_vibrancy(
            _window,
            NSVisualEffectMaterial::HUDWindow,
            Some(NSVisualEffectState::Active),
            Some(12.0),
        );
    }

    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic, apply_mica};
        let _ = apply_mica(_window, Some(true))
            .or_else(|_| apply_acrylic(_window, Some((8, 9, 10, 180))));
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(main_win) = app.get_webview_window("main") {
                apply_window_vibrancy(&main_win);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            probe_path,
            get_matrix,
            get_tools_for_category,
            get_actions,
            get_targets,
            pick_files,
            execute_conversion,
            minimize_window,
            toggle_maximize_window,
            close_window,
            start_window_dragging,
        ])
        .run(tauri::generate_context!())
        .expect("error while running shaft application");
}
