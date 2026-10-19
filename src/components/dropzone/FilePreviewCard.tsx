import type { FC } from "react";
import { FileProbeResult } from "../../types";
import { FileText, Image, Video, Music, Archive, Sparkles, X, ArrowRight } from "lucide-react";

interface FilePreviewCardProps {
  probe: FileProbeResult;
  onSelectTarget: (targetFormat: string) => void;
  onOpenWheel: () => void;
  onRemove: () => void;
}

export const FilePreviewCard: FC<FilePreviewCardProps> = ({
  probe,
  onSelectTarget,
  onOpenWheel,
  onRemove,
}) => {
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case "image":
        return <Image className="w-4 h-4 text-sky-400" />;
      case "video":
        return <Video className="w-4 h-4 text-purple-400" />;
      case "audio":
        return <Music className="w-4 h-4 text-amber-400" />;
      case "archive":
        return <Archive className="w-4 h-4 text-emerald-400" />;
      default:
        return <FileText className="w-4 h-4 text-brand-light" />;
    }
  };

  return (
    <div className="glass-panel rounded-xl p-4 flex flex-col gap-3 transition-all hover:border-white/15">
      {/* File Info Header */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          <div className="w-10 h-10 rounded-lg bg-white/5 border border-white/10 flex items-center justify-center shrink-0">
            {getCategoryIcon(probe.category)}
          </div>
          <div className="min-w-0">
            <div className="text-xs font-semibold text-white/90 truncate max-w-sm">
              {probe.file_name}
            </div>
            <div className="flex items-center gap-2 mt-0.5 text-[11px] font-mono text-white/40">
              <span className="uppercase text-brand-light font-bold">
                .{probe.detected_format}
              </span>
              <span>•</span>
              <span>{formatBytes(probe.file_size_bytes)}</span>
              <span>•</span>
              <span className="capitalize">{probe.category}</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <button
            onClick={onOpenWheel}
            className="flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-medium bg-brand/20 text-brand-light border border-brand/30 hover:bg-brand/30 transition-all shadow-sm"
          >
            <Sparkles className="w-3 h-3" />
            <span>Radial Wheel</span>
          </button>
          <button
            onClick={onRemove}
            className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-white/10 text-white/40 hover:text-white transition-colors"
            title="Remove file"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Target Format Chips */}
      {probe.available_targets.length > 0 && (
        <div className="flex flex-col gap-1.5 pt-2 border-t border-hairline">
          <div className="text-[10px] uppercase font-mono tracking-wider text-white/40">
            Convert To:
          </div>
          <div className="flex flex-wrap gap-1.5">
            {probe.available_targets.map((tgt) => (
              <button
                key={tgt}
                onClick={() => onSelectTarget(tgt)}
                className="group flex items-center gap-1 px-2 py-0.5 rounded-md bg-white/5 hover:bg-brand/25 border border-white/10 hover:border-brand/40 text-[11px] font-mono uppercase text-white/70 hover:text-brand-light transition-all"
              >
                <span>{tgt}</span>
                <ArrowRight className="w-2.5 h-2.5 opacity-0 group-hover:opacity-100 transition-opacity" />
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
