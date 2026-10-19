export type FileCategory =
  | "image"
  | "audio"
  | "video"
  | "pdf"
  | "presentation"
  | "spreadsheet"
  | "document"
  | "ebook"
  | "subtitle"
  | "archive"
  | "unknown";

export interface ToolInfo {
  id: string;
  label: string;
  description: string;
  category: FileCategory;
  icon_name: string;
}

export interface ActionInfo {
  id: string;
  label: string;
  description: string;
  icon_name: string;
}

export interface ExtendedMetadata {
  width?: number;
  height?: number;
  duration_seconds?: number;
  sample_rate?: number;
  channels?: number;
  page_count?: number;
  mime_type?: string;
}

export interface FileProbeResult {
  file_path: string;
  file_name: string;
  file_size_bytes: number;
  detected_format: string;
  category: FileCategory;
  available_targets: string[];
  applicable_tools: ToolInfo[];
  metadata: ExtendedMetadata;
}

export type TaskStatus = "queued" | "processing" | "completed" | "failed" | "cancelled";

export interface ConversionTask {
  id: string;
  source_path: string;
  source_name: string;
  source_format: string;
  target_format: string;
  category: FileCategory;
  status: TaskStatus;
  progress: number;
  speed_bytes_per_sec?: number;
  eta_seconds?: number;
  error_message?: string;
  output_path?: string;
}

export interface TaskProgressEvent {
  task_id: string;
  status: TaskStatus;
  progress: number;
  bytes_processed: number;
  total_bytes: number;
  speed_bytes_per_sec?: number;
  eta_seconds?: number;
  error_message?: string;
  output_path?: string;
}
