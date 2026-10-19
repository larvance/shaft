// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use shaft_core::{
    get_all_conversions, get_applicable_tools, get_available_targets,
    get_file_actions, probe_file, ActionInfo, ConversionDAG, ConversionRoute,
    ConversionTask, FileCategory, FileProbeResult, TaskQueueManager, ToolInfo,
};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

static QUEUE_MANAGER: OnceLock<TaskQueueManager> = OnceLock::new();
static CONVERSION_DAG: OnceLock<ConversionDAG> = OnceLock::new();

fn get_queue() -> &'static TaskQueueManager {
    QUEUE_MANAGER.get_or_init(TaskQueueManager::new)
}

fn get_dag() -> &'static ConversionDAG {
    CONVERSION_DAG.get_or_init(ConversionDAG::new)
}

#[tauri::command]
fn probe_path(path: String) -> Result<FileProbeResult, String> {
    probe_file(path)
}

#[tauri::command]
fn inspect_archive_entries(path: String) -> Result<shaft_archive::ArchiveMetadata, String> {
    shaft_archive::inspect_archive(path)
}

#[tauri::command]
fn get_matrix() -> HashMap<String, Vec<String>> {
    get_all_conversions()
}

#[tauri::command]
fn get_route(source: String, target: String) -> Result<ConversionRoute, String> {
    get_dag()
        .find_shortest_route(&source, &target)
        .ok_or_else(|| format!("No conversion route found from {} to {}", source, target))
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
fn submit_task(task: ConversionTask) -> Result<String, String> {
    let task_id = get_queue().submit_task(task);
    Ok(task_id)
}

#[tauri::command]
fn submit_batch(tasks: Vec<ConversionTask>) -> Result<Vec<String>, String> {
    let ids = get_queue().submit_batch(tasks);
    Ok(ids)
}

#[tauri::command]
fn cancel_task(task_id: String) -> Result<bool, String> {
    Ok(get_queue().cancel_task(&task_id))
}

#[tauri::command]
fn cancel_all_tasks() -> Result<(), String> {
    get_queue().cancel_all();
    Ok(())
}

#[tauri::command]
fn get_all_tasks() -> Vec<ConversionTask> {
    get_queue().get_all_tasks()
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

fn setup_event_emitter(app: &AppHandle) {
    let handle = app.clone();
    let callback = Arc::new(move |event| {
        let _ = handle.emit("shaft://task-progress", event);
    });
    let queue = TaskQueueManager::with_progress_callback(callback);

    // Register pure safe image transcoding engine
    queue.register_transcoder(
        FileCategory::Image,
        Arc::new(|src, dst, fmt| {
            let opts = shaft_image::ImageTranscodeOptions::default();
            shaft_image::transcode_image(src, dst, fmt, &opts)
        }),
    );

    // Register audio transcoding engine
    queue.register_transcoder(
        FileCategory::Audio,
        Arc::new(|src, dst, fmt| {
            let opts = shaft_audio::AudioTranscodeOptions::default();
            shaft_audio::transcode_audio(src, dst, fmt, &opts)
        }),
    );

    // Register video transcoding engine
    queue.register_transcoder(
        FileCategory::Video,
        Arc::new(|src, dst, fmt| {
            let opts = shaft_video::VideoTranscodeOptions::default();
            shaft_video::transcode_video(src, dst, fmt, &opts)
        }),
    );

    // Register spreadsheet transcoding engine (XLSX, XLS, ODS, CSV, TSV)
    queue.register_transcoder(
        FileCategory::Spreadsheet,
        Arc::new(|src, dst, fmt| {
            shaft_document::transcode_spreadsheet(src, dst, fmt)
        }),
    );

    // Register PDF extraction engine
    queue.register_transcoder(
        FileCategory::Pdf,
        Arc::new(|src, dst, fmt| {
            shaft_document::transcode_pdf(src, dst, fmt)
        }),
    );

    // Register document compiler engine (Markdown, HTML, TXT, DOCX)
    queue.register_transcoder(
        FileCategory::Document,
        Arc::new(|src, dst, fmt| {
            shaft_document::transcode_document(src, dst, fmt)
        }),
    );

    queue.register_transcoder(
        FileCategory::Ebook,
        Arc::new(|src, dst, fmt| {
            shaft_document::transcode_document(src, dst, fmt)
        }),
    );

    // Register subtitle conversion engine (SRT, VTT)
    queue.register_transcoder(
        FileCategory::Subtitle,
        Arc::new(|src, dst, fmt| {
            shaft_document::transcode_subtitle(src, dst, fmt)
        }),
    );

    // Register archive engine (ZIP, TAR, GZ, TGZ)
    queue.register_transcoder(
        FileCategory::Archive,
        Arc::new(|src, dst, fmt| {
            shaft_archive::transcode_archive(src, dst, fmt)
        }),
    );

    let _ = QUEUE_MANAGER.set(queue);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            setup_event_emitter(app.handle());
            if let Some(main_win) = app.get_webview_window("main") {
                apply_window_vibrancy(&main_win);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            probe_path,
            inspect_archive_entries,
            get_matrix,
            get_route,
            get_tools_for_category,
            get_actions,
            get_targets,
            pick_files,
            submit_task,
            submit_batch,
            cancel_task,
            cancel_all_tasks,
            get_all_tasks,
            minimize_window,
            toggle_maximize_window,
            close_window,
            start_window_dragging,
        ])
        .run(tauri::generate_context!())
        .expect("error while running shaft application");
}
