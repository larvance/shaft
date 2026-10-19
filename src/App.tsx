import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Titlebar } from "./components/layout/Titlebar";
import { Footer } from "./components/layout/Footer";
import { DropZone } from "./components/dropzone/DropZone";
import { FilePreviewCard } from "./components/dropzone/FilePreviewCard";
import { RadialWheel } from "./components/wheel/RadialWheel";
import { TaskQueue } from "./components/queue/TaskQueue";
import { DragGesturePill } from "./components/gesture/DragGesturePill";
import { useModifiers } from "./hooks/useModifiers";
import { useQueueStore } from "./stores/useQueueStore";
import { ConversionTask, FileProbeResult } from "./types";
import { WheelItem, WheelMode } from "./types/wheel";

export function App() {
  const { shift, alt, ctrl, meta, activeMode: keyboardMode } = useModifiers();
  const [explicitMode, setExplicitMode] = useState<WheelMode | null>(null);

  // Active mode is either keyboard-driven or explicitly toggled via UI
  const activeMode: WheelMode | null = explicitMode || keyboardMode;

  const [activeFiles, setActiveFiles] = useState<FileProbeResult[]>([]);
  const [allConversions, setAllConversions] = useState<Record<string, string[]>>({});
  const [isWheelModalOpen, setIsWheelModalOpen] = useState(false);

  // Global drag gesture pill tracking
  const [isSystemDragging, setIsSystemDragging] = useState(false);
  const [dragCursor, setDragCursor] = useState<{ x: number; y: number } | null>(null);
  const [dragFileCount, setDragFileCount] = useState(1);
  const dragCounter = useRef(0);

  // Connect to global batch queue store
  const {
    tasks,
    initEventListener,
    submitTask,
    cancelTask,
    cancelAll,
    clearCompleted,
  } = useQueueStore();

  // Initialize Tauri event listener on mount
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    initEventListener().then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [initEventListener]);

  // Load format conversion matrix from Rust backend
  useEffect(() => {
    async function loadMatrix() {
      try {
        const matrix = await invoke<Record<string, string[]>>("get_matrix");
        setAllConversions(matrix);
      } catch (e) {
        console.warn("Failed to load conversion matrix via Tauri, using fallback", e);
      }
    }
    loadMatrix();
  }, []);

  const totalConversionsCount = useMemo(() => {
    let sum = 0;
    for (const key of Object.keys(allConversions)) {
      sum += allConversions[key].length;
    }
    return sum > 0 ? sum : 275;
  }, [allConversions]);

  // Handle files dropped or selected
  const handleFilesSelected = useCallback(async (filePaths: string[]) => {
    for (const path of filePaths) {
      try {
        const probe = await invoke<FileProbeResult>("probe_path", { path });
        setActiveFiles((prev) => {
          if (prev.some((f) => f.file_path === probe.file_path)) {
            return prev;
          }
          return [probe, ...prev];
        });
      } catch (e) {
        console.warn("Tauri probe_path failed, using fallback parser", e);
        // Fallback probe
        const fileName = path.split("/").pop() || path;
        const ext = fileName.split(".").pop()?.toLowerCase() || "png";
        const fallbackProbe: FileProbeResult = {
          file_path: path,
          file_name: fileName,
          file_size_bytes: 1024 * 1024 * 2.4,
          detected_format: ext,
          category: "image",
          available_targets: ["jpg", "png", "webp", "heic", "tiff", "avif", "pdf"],
          applicable_tools: [],
          metadata: { width: 1920, height: 1080 },
        };
        setActiveFiles((prev) => [fallbackProbe, ...prev]);
      }
    }
  }, []);

  // Primary active file for Radial Wheel context
  const primaryFile = activeFiles.length > 0 ? activeFiles[0] : null;

  // Build items for Radial Wheel depending on activeMode
  const wheelItems: WheelItem[] = useMemo(() => {
    if (!primaryFile) {
      // Default demo wheel when no file is ingested
      return [
        { id: "jpg", label: "JPG", payload: "jpg" },
        { id: "png", label: "PNG", payload: "png" },
        { id: "webp", label: "WEBP", payload: "webp" },
        { id: "pdf", label: "PDF", payload: "pdf" },
        { id: "avif", label: "AVIF", payload: "avif" },
        { id: "heic", label: "HEIC", payload: "heic" },
      ];
    }

    if (activeMode === "tool") {
      if (primaryFile.applicable_tools.length > 0) {
        return primaryFile.applicable_tools.map((t) => ({
          id: t.id,
          label: t.label,
          icon: t.icon_name,
          category: t.category,
          payload: t,
        }));
      }
      return [
        { id: "compress", label: "Lossless Compress", icon: "Minimize2", payload: "compress" },
        { id: "metadata", label: "Scrub Metadata", icon: "ShieldCheck", payload: "metadata" },
        { id: "blur", label: "Face Blur", icon: "EyeOff", payload: "blur" },
        { id: "ocr", label: "OCR Text", icon: "ScanText", payload: "ocr" },
      ];
    }

    if (activeMode === "action") {
      return [
        { id: "share", label: "Share", icon: "Share2", payload: "share" },
        { id: "quicklook", label: "QuickLook", icon: "Maximize2", payload: "quicklook" },
        { id: "copy", label: "Copy File", icon: "Copy", payload: "copy" },
        { id: "path", label: "Copy Path", icon: "Link", payload: "path" },
        { id: "terminal", label: "Terminal", icon: "Terminal", payload: "terminal" },
      ];
    }

    // Default: format wheel
    if (primaryFile.available_targets.length > 0) {
      return primaryFile.available_targets.map((tgt) => ({
        id: tgt,
        label: tgt.toUpperCase(),
        payload: tgt,
      }));
    }

    return [
      { id: "png", label: "PNG", payload: "png" },
      { id: "jpg", label: "JPG", payload: "jpg" },
      { id: "webp", label: "WEBP", payload: "webp" },
      { id: "pdf", label: "PDF", payload: "pdf" },
    ];
  }, [primaryFile, activeMode]);

  // Execute conversion task via queue store
  const executeConversion = useCallback(
    async (file: FileProbeResult, targetFormat: string) => {
      const taskId = `task_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
      const newTask: ConversionTask = {
        id: taskId,
        source_path: file.file_path,
        source_name: file.file_name,
        source_format: file.detected_format,
        target_format: targetFormat,
        category: file.category,
        status: "queued",
        progress: 0.0,
      };

      await submitTask(newTask);
    },
    [submitTask]
  );

  const handleSelectWheelItem = async (item: WheelItem) => {
    if (!primaryFile) return;

    if (activeMode === "format") {
      executeConversion(primaryFile, item.id);
    } else if (activeMode === "tool") {
      try {
        await invoke("execute_media_tool", {
          path: primaryFile.file_path,
          toolId: item.id,
          options: {},
        });
      } catch (e) {
        console.error("Tool execution failed:", e);
      }
    } else if (activeMode === "action") {
      try {
        if (item.id === "path") {
          await navigator.clipboard.writeText(primaryFile.file_path);
        } else {
          await invoke("execute_file_action", {
            path: primaryFile.file_path,
            actionId: item.id,
          });
        }
      } catch (e) {
        console.error("Action execution failed:", e);
      }
    }

    setIsWheelModalOpen(false);
    setExplicitMode(null);
  };

  // Auto-open radial wheel modal when modifier key is pressed while files are loaded
  useEffect(() => {
    if (activeFiles.length > 0 && (shift || alt || ctrl || meta)) {
      setIsWheelModalOpen(true);
    }
  }, [shift, alt, ctrl, meta, activeFiles.length]);

  // Global window drag event listeners for DragGesturePill
  useEffect(() => {
    const handleDragEnter = (e: DragEvent) => {
      e.preventDefault();
      dragCounter.current += 1;
      if (e.dataTransfer && e.dataTransfer.types.includes("Files")) {
        setIsSystemDragging(true);
        setDragCursor({ x: e.clientX, y: e.clientY });
        const count = e.dataTransfer.items?.length || e.dataTransfer.files?.length || 1;
        setDragFileCount(count > 0 ? count : 1);
      }
    };

    const handleDragOver = (e: DragEvent) => {
      e.preventDefault();
      setDragCursor({ x: e.clientX, y: e.clientY });
      if (!isSystemDragging && e.dataTransfer && e.dataTransfer.types.includes("Files")) {
        setIsSystemDragging(true);
      }
    };

    const handleDragLeave = (e: DragEvent) => {
      e.preventDefault();
      dragCounter.current -= 1;
      if (dragCounter.current <= 0) {
        dragCounter.current = 0;
        setIsSystemDragging(false);
        setDragCursor(null);
      }
    };

    const handleDrop = (e: DragEvent) => {
      e.preventDefault();
      dragCounter.current = 0;
      setIsSystemDragging(false);
      setDragCursor(null);

      if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
        const paths = Array.from(e.dataTransfer.files).map(
          (f) => (f as unknown as { path?: string }).path || f.name
        );
        handleFilesSelected(paths);
      }
    };

    window.addEventListener("dragenter", handleDragEnter);
    window.addEventListener("dragover", handleDragOver);
    window.addEventListener("dragleave", handleDragLeave);
    window.addEventListener("drop", handleDrop);

    return () => {
      window.removeEventListener("dragenter", handleDragEnter);
      window.removeEventListener("dragover", handleDragOver);
      window.removeEventListener("dragleave", handleDragLeave);
      window.removeEventListener("drop", handleDrop);
    };
  }, [handleFilesSelected, isSystemDragging]);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-obsidian-950/80 backdrop-blur-3xl text-white">
      {/* Draggable Titlebar */}
      <Titlebar
        activeMode={activeMode}
        onModeSelect={(mode) => {
          setExplicitMode(mode);
          if (mode) setIsWheelModalOpen(true);
        }}
      />

      {/* Main Workspace Body */}
      <main className="flex-1 overflow-y-auto p-4 flex flex-col gap-4 max-w-4xl w-full mx-auto">
        {/* Drop Zone */}
        <DropZone onFilesSelected={handleFilesSelected} />

        {/* Selected Files List */}
        {activeFiles.length > 0 && (
          <div className="flex flex-col gap-2 mt-2">
            <div className="text-xs font-semibold text-white/80 px-1">
              Active Files ({activeFiles.length})
            </div>
            {activeFiles.map((file) => (
              <FilePreviewCard
                key={file.file_path}
                probe={file}
                onSelectTarget={(target) => executeConversion(file, target)}
                onOpenWheel={() => {
                  setExplicitMode("format");
                  setIsWheelModalOpen(true);
                }}
                onRemove={() =>
                  setActiveFiles((prev) =>
                    prev.filter((f) => f.file_path !== file.file_path)
                  )
                }
              />
            ))}
          </div>
        )}

        {/* Batch Queue */}
        <TaskQueue
          tasks={tasks}
          onCancelTask={cancelTask}
          onCancelAll={cancelAll}
          onClearCompleted={clearCompleted}
          onRevealOutput={(path) => {
            console.log("Reveal in file manager:", path);
          }}
        />
      </main>

      {/* Floating Radial Wheel Modal Overlay */}
      {isWheelModalOpen && (
        <div
          onClick={() => {
            setIsWheelModalOpen(false);
            setExplicitMode(null);
          }}
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-md animate-in fade-in duration-150"
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="glass-panel-elevated rounded-3xl p-4 shadow-2xl animate-in zoom-in-95 duration-150"
          >
            <RadialWheel
              items={wheelItems}
              mode={activeMode || "format"}
              sourceFileName={primaryFile?.file_name}
              onSelect={handleSelectWheelItem}
              onCancel={() => {
                setIsWheelModalOpen(false);
                setExplicitMode(null);
              }}
            />
          </div>
        </div>
      )}

      {/* Global Drag Gesture Pill */}
      {isSystemDragging && dragCursor && (
        <DragGesturePill
          x={dragCursor.x}
          y={dragCursor.y}
          fileCount={dragFileCount}
          formatBadge={primaryFile?.detected_format}
          activeModifier={activeMode || undefined}
        />
      )}

      {/* Footer Status Bar */}
      <Footer
        activeMode={activeMode}
        totalConversions={totalConversionsCount}
      />
    </div>
  );
}
