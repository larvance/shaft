import { FC } from "react";
import { Files } from "lucide-react";

interface DragGesturePillProps {
  x: number;
  y: number;
  fileCount: number;
  formatBadge?: string;
  activeModifier?: "format" | "tool" | "action";
}

export const DragGesturePill: FC<DragGesturePillProps> = ({
  x,
  y,
  fileCount,
  formatBadge,
  activeModifier,
}) => {
  return (
    <div
      style={{
        left: `${x}px`,
        top: `${y}px`,
      }}
      className="fixed pointer-events-none z-50 transform -translate-x-1/2 -translate-y-12 transition-all duration-75 ease-out select-none"
    >
      <div className="flex items-center space-x-2.5 px-3.5 py-2 rounded-full bg-obsidian-950/92 backdrop-blur-2xl border border-white/15 shadow-[0_12px_32px_rgba(0,0,0,0.65)] text-white">
        {/* Glow indicator dot */}
        <div className="relative flex items-center justify-center">
          <span className="w-2 h-2 rounded-full bg-brand-light animate-ping absolute" />
          <span className="w-2 h-2 rounded-full bg-brand-light relative" />
        </div>

        {/* File count and icon */}
        <div className="flex items-center space-x-1.5 font-medium text-xs">
          <Files className="w-3.5 h-3.5 text-white/70" />
          <span>
            {fileCount} {fileCount === 1 ? "File" : "Files"}
          </span>
        </div>

        {/* Format badge */}
        {formatBadge && (
          <span className="px-1.5 py-0.5 rounded text-[10px] font-mono font-semibold uppercase bg-white/10 text-white/90 border border-white/10">
            {formatBadge}
          </span>
        )}

        {/* Divider */}
        <div className="w-[1px] h-3 bg-white/15" />

        {/* Modifier shortcuts */}
        <div className="flex items-center space-x-2 text-[10px] text-white/60">
          <span
            className={`transition-colors duration-150 ${
              activeModifier === "format" ? "text-brand-light font-semibold" : ""
            }`}
          >
            <kbd className="px-1 py-0.5 bg-white/5 rounded border border-white/10 text-[9px] mr-0.5">Shift</kbd>
            Formats
          </span>
          <span
            className={`transition-colors duration-150 ${
              activeModifier === "tool" ? "text-amber-300 font-semibold" : ""
            }`}
          >
            <kbd className="px-1 py-0.5 bg-white/5 rounded border border-white/10 text-[9px] mr-0.5">Alt</kbd>
            Tools
          </span>
          <span
            className={`transition-colors duration-150 ${
              activeModifier === "action" ? "text-emerald-300 font-semibold" : ""
            }`}
          >
            <kbd className="px-1 py-0.5 bg-white/5 rounded border border-white/10 text-[9px] mr-0.5">Ctrl</kbd>
            Actions
          </span>
        </div>
      </div>
    </div>
  );
};
