import { useState, type FC, type DragEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { UploadCloud, FilePlus, Sparkles } from "lucide-react";

interface DropZoneProps {
  onFilesSelected: (filePaths: string[]) => void;
}

export const DropZone: FC<DropZoneProps> = ({ onFilesSelected }) => {
  const [isDragOver, setIsDragOver] = useState(false);

  const handlePickFiles = async () => {
    try {
      const paths = await invoke<string[]>("pick_files");
      if (paths && paths.length > 0) {
        onFilesSelected(paths);
      }
    } catch (e) {
      console.warn("Tauri pick_files fallback to HTML input", e);
      // Fallback HTML file picker
      const input = document.createElement("input");
      input.type = "file";
      input.multiple = true;
      input.onchange = (ev) => {
        const files = (ev.target as HTMLInputElement).files;
        if (files && files.length > 0) {
          const names = Array.from(files).map((f) => f.name);
          onFilesSelected(names);
        }
      };
      input.click();
    }
  };

  const onDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  };

  const onDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  };

  const onDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  };

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const paths = Array.from(e.dataTransfer.files).map(
        (f) => (f as unknown as { path?: string }).path || f.name
      );
      onFilesSelected(paths);
    }
  };

  return (
    <div
      onDragEnter={onDragEnter}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onClick={handlePickFiles}
      className={`relative w-full rounded-2xl border transition-all duration-200 cursor-pointer p-8 flex flex-col items-center justify-center text-center ${
        isDragOver
          ? "bg-brand/10 border-brand-light/60 shadow-[0_0_24px_rgba(94,106,210,0.25)]"
          : "bg-obsidian-900/40 hover:bg-obsidian-900/60 border-dashed border-white/10 hover:border-white/20"
      }`}
    >
      <div className="w-12 h-12 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center mb-3 shadow-inner group-hover:scale-105 transition-transform">
        {isDragOver ? (
          <FilePlus className="w-6 h-6 text-brand-light animate-bounce" />
        ) : (
          <UploadCloud className="w-6 h-6 text-white/60" />
        )}
      </div>

      <div className="text-sm font-medium text-white/90">
        Drop files here to convert
      </div>
      <div className="text-xs text-white/40 mt-1 max-w-sm">
        Supports 275+ format conversions across Images, Audio, Video, PDF, Documents, Spreadsheets, and Archives
      </div>

      <div className="mt-4 flex items-center gap-2 text-[10px] text-white/50 bg-white/5 border border-white/10 px-2.5 py-1 rounded-full">
        <Sparkles className="w-3 h-3 text-brand-light" />
        <span>Hold Shift / Alt / Ctrl for instant Radial Wheels</span>
      </div>
    </div>
  );
};
