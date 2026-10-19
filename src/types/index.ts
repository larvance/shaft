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

export interface FileProbeResult {
  file_path: string;
  file_name: string;
  file_size_bytes: number;
  detected_format: string;
  category: FileCategory;
  available_targets: string[];
  applicable_tools: ToolInfo[];
}

export type TaskStatus = "queued" | "processing" | "completed" | "failed";

export interface ConversionTask {
  id: string;
  source_path: string;
  source_name: string;
  source_format: string;
  target_format: string;
  category: FileCategory;
  status: TaskStatus;
  progress: number;
  error_message?: string;
  output_path?: string;
}
