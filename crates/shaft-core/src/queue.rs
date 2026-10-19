use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use crate::types::{ConversionTask, TaskProgressEvent, TaskStatus};

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

pub type ProgressCallback = Arc<dyn Fn(TaskProgressEvent) + Send + Sync>;

#[derive(Clone)]
pub struct TaskQueueManager {
    tasks: Arc<RwLock<HashMap<String, ConversionTask>>>,
    cancellations: Arc<RwLock<HashMap<String, CancellationToken>>>,
    progress_callback: Option<ProgressCallback>,
}

impl Default for TaskQueueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueueManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            cancellations: Arc::new(RwLock::new(HashMap::new())),
            progress_callback: None,
        }
    }

    pub fn with_progress_callback(callback: ProgressCallback) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            cancellations: Arc::new(RwLock::new(HashMap::new())),
            progress_callback: Some(callback),
        }
    }

    pub fn submit_task(&self, task: ConversionTask) -> String {
        let task_id = task.id.clone();
        let token = CancellationToken::new();

        {
            let mut tasks_lock = self.tasks.write().unwrap();
            tasks_lock.insert(task_id.clone(), task.clone());
            let mut cancel_lock = self.cancellations.write().unwrap();
            cancel_lock.insert(task_id.clone(), token.clone());
        }

        // Spawn async execution
        let manager_clone = self.clone();
        rayon::spawn(move || {
            manager_clone.execute_task_internal(task, token);
        });

        task_id
    }

    pub fn submit_batch(&self, tasks: Vec<ConversionTask>) -> Vec<String> {
        tasks.into_iter().map(|t| self.submit_task(t)).collect()
    }

    pub fn cancel_task(&self, task_id: &str) -> bool {
        let cancel_lock = self.cancellations.read().unwrap();
        if let Some(token) = cancel_lock.get(task_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn cancel_all(&self) {
        let cancel_lock = self.cancellations.read().unwrap();
        for token in cancel_lock.values() {
            token.cancel();
        }
    }

    pub fn get_task(&self, task_id: &str) -> Option<ConversionTask> {
        let tasks_lock = self.tasks.read().unwrap();
        tasks_lock.get(task_id).cloned()
    }

    pub fn get_all_tasks(&self) -> Vec<ConversionTask> {
        let tasks_lock = self.tasks.read().unwrap();
        tasks_lock.values().cloned().collect()
    }

    pub fn clear_completed(&mut self) {
        let mut tasks_lock = self.tasks.write().unwrap();
        tasks_lock.retain(|_, t| t.status != TaskStatus::Completed && t.status != TaskStatus::Cancelled);
    }

    fn execute_task_internal(&self, mut task: ConversionTask, token: CancellationToken) {
        let start_time = Instant::now();
        let src = Path::new(&task.source_path);

        if token.is_cancelled() {
            task.status = TaskStatus::Cancelled;
            self.emit_progress(&task, 0, 0, None, None);
            self.update_task_state(task);
            return;
        }

        if !src.exists() {
            task.status = TaskStatus::Failed;
            task.error_message = Some("Source file does not exist".into());
            self.emit_progress(&task, 0, 0, None, None);
            self.update_task_state(task);
            return;
        }

        let total_bytes = match std::fs::metadata(src) {
            Ok(m) => m.len(),
            Err(e) => {
                task.status = TaskStatus::Failed;
                task.error_message = Some(e.to_string());
                self.emit_progress(&task, 0, 0, None, None);
                self.update_task_state(task);
                return;
            }
        };

        // Transition to Processing
        task.status = TaskStatus::Processing;
        task.progress = 0.05;
        self.emit_progress(&task, 0, total_bytes, None, None);
        self.update_task_state(task.clone());

        let parent = src.parent().unwrap_or_else(|| Path::new("."));
        let stem = src
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        let out_name = format!("{}_converted.{}", stem, task.target_format);
        let out_path = parent.join(out_name);

        // Simulated progress passes with cancellation checkpoints
        let steps = 5;
        for i in 1..=steps {
            if token.is_cancelled() {
                task.status = TaskStatus::Cancelled;
                task.error_message = Some("Task cancelled by operator".into());
                self.emit_progress(&task, (total_bytes * i as u64) / steps as u64, total_bytes, None, None);
                self.update_task_state(task);
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(40));

            let progress = (i as f32) / (steps as f32);
            let processed_bytes = ((total_bytes as f32) * progress) as u64;
            let elapsed = start_time.elapsed().as_secs_f32();
            let speed = if elapsed > 0.0 {
                Some((processed_bytes as f32 / elapsed) as u64)
            } else {
                None
            };
            let eta = if progress > 0.0 && progress < 1.0 {
                Some((elapsed / progress) * (1.0 - progress))
            } else {
                None
            };

            task.progress = progress;
            self.emit_progress(&task, processed_bytes, total_bytes, speed, eta);
            self.update_task_state(task.clone());
        }

        // Output generation pass
        if let Err(e) = std::fs::copy(src, &out_path) {
            task.status = TaskStatus::Failed;
            task.error_message = Some(format!("IO error: {}", e));
        } else {
            task.status = TaskStatus::Completed;
            task.progress = 1.0;
            task.output_path = Some(out_path.to_string_lossy().to_string());
        }

        self.emit_progress(&task, total_bytes, total_bytes, None, None);
        self.update_task_state(task);
    }

    fn update_task_state(&self, task: ConversionTask) {
        let mut tasks_lock = self.tasks.write().unwrap();
        tasks_lock.insert(task.id.clone(), task);
    }

    fn emit_progress(
        &self,
        task: &ConversionTask,
        bytes_processed: u64,
        total_bytes: u64,
        speed: Option<u64>,
        eta: Option<f32>,
    ) {
        if let Some(ref cb) = self.progress_callback {
            cb(TaskProgressEvent {
                task_id: task.id.clone(),
                status: task.status,
                progress: task.progress,
                bytes_processed,
                total_bytes,
                speed_bytes_per_sec: speed,
                eta_seconds: eta,
                error_message: task.error_message.clone(),
                output_path: task.output_path.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use crate::types::FileCategory;

    #[test]
    fn test_task_queue_execution() {
        let temp_dir = std::env::temp_dir();
        let test_src = temp_dir.join("shaft_test_input.png");
        {
            let mut f = std::fs::File::create(&test_src).unwrap();
            f.write_all(b"sample test content for shaft").unwrap();
        }

        let manager = TaskQueueManager::new();
        let task = ConversionTask {
            id: "task_test_1".into(),
            source_path: test_src.to_string_lossy().to_string(),
            source_name: "shaft_test_input.png".into(),
            source_format: "png".into(),
            target_format: "webp".into(),
            category: FileCategory::Image,
            status: TaskStatus::Queued,
            progress: 0.0,
            error_message: None,
            output_path: None,
        };

        manager.submit_task(task);

        // Wait for completion
        let mut completed = false;
        for _ in 0..50 {
            std::thread::sleep(std::time::Duration::from_millis(20));
            if let Some(t) = manager.get_task("task_test_1") {
                if t.status == TaskStatus::Completed {
                    completed = true;
                    assert_eq!(t.progress, 1.0);
                    assert!(t.output_path.is_some());
                    break;
                }
            }
        }

        assert!(completed, "Task should complete successfully");
        let _ = std::fs::remove_file(&test_src);
    }

    #[test]
    fn test_task_cancellation() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
}
