import { useState, useEffect, type FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Minus, Square, X, Layers, Sparkles, Sliders, Zap } from "lucide-react";
import { WheelMode } from "../../types/wheel";

interface TitlebarProps {
  activeMode: WheelMode | null;
  onModeSelect?: (mode: WheelMode | null) => void;
}

export const Titlebar: FC<TitlebarProps> = ({ activeMode, onModeSelect }) => {
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    // Check if running inside Tauri
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      setIsTauri(true);
    }
  }, []);

  const handleMinimize = async () => {
    if (isTauri) {
      try {
        await invoke("minimize_window");
      } catch (e) {
        console.error(e);
      }
    }
  };

  const handleMaximize = async () => {
    if (isTauri) {
      try {
        await invoke("toggle_maximize_window");
      } catch (e) {
        console.error(e);
      }
    }
  };

  const handleClose = async () => {
    if (isTauri) {
      try {
        await invoke("close_window");
      } catch (e) {
        console.error(e);
      }
    }
  };

  return (
    <header
      data-tauri-drag-region
      className="h-10 flex items-center justify-between px-3 select-none bg-obsidian-950/40 border-b border-hairline backdrop-blur-md z-50 shrink-0"
    >
      {/* Brand Identity */}
      <div className="flex items-center gap-2.5 pointer-events-auto">
        <div className="w-5 h-5 rounded-md bg-brand/20 border border-brand/40 flex items-center justify-center">
          <Zap className="w-3 h-3 text-brand-light" />
        </div>
        <span className="font-semibold text-xs tracking-tight text-white/90">
          Shaft
        </span>
        <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-white/40">
          v0.1.0
        </span>
      </div>

      {/* Mode Switcher Buttons */}
      <div className="flex items-center gap-1 pointer-events-auto bg-obsidian-900/60 p-0.5 rounded-lg border border-hairline">
        <button
          onClick={() => onModeSelect?.(activeMode === "format" ? null : "format")}
          className={`flex items-center gap-1.5 px-2 py-1 rounded text-[11px] font-medium transition-all ${
            activeMode === "format"
              ? "bg-brand/25 text-brand-light border border-brand/40 shadow-sm"
              : "text-white/60 hover:text-white/90 hover:bg-white/5 border border-transparent"
          }`}
        >
          <Layers className="w-3 h-3" />
          <span>Formats</span>
          <kbd className="text-[9px] font-mono px-1 py-0.2 rounded bg-black/40 text-white/50 border border-white/5">
            Shift
          </kbd>
        </button>

        <button
          onClick={() => onModeSelect?.(activeMode === "tool" ? null : "tool")}
          className={`flex items-center gap-1.5 px-2 py-1 rounded text-[11px] font-medium transition-all ${
            activeMode === "tool"
              ? "bg-amber-500/25 text-amber-300 border border-amber-500/40 shadow-sm"
              : "text-white/60 hover:text-white/90 hover:bg-white/5 border border-transparent"
          }`}
        >
          <Sliders className="w-3 h-3" />
          <span>Tools</span>
          <kbd className="text-[9px] font-mono px-1 py-0.2 rounded bg-black/40 text-white/50 border border-white/5">
            Alt
          </kbd>
        </button>

        <button
          onClick={() => onModeSelect?.(activeMode === "action" ? null : "action")}
          className={`flex items-center gap-1.5 px-2 py-1 rounded text-[11px] font-medium transition-all ${
            activeMode === "action"
              ? "bg-emerald-500/25 text-emerald-300 border border-emerald-500/40 shadow-sm"
              : "text-white/60 hover:text-white/90 hover:bg-white/5 border border-transparent"
          }`}
        >
          <Sparkles className="w-3 h-3" />
          <span>Actions</span>
          <kbd className="text-[9px] font-mono px-1 py-0.2 rounded bg-black/40 text-white/50 border border-white/5">
            Ctrl
          </kbd>
        </button>
      </div>

      {/* Window Controls */}
      <div className="flex items-center gap-1 pointer-events-auto">
        <button
          onClick={handleMinimize}
          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-white/10 text-white/60 hover:text-white transition-colors"
          title="Minimize"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={handleMaximize}
          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-white/10 text-white/60 hover:text-white transition-colors"
          title="Maximize"
        >
          <Square className="w-3 h-3" />
        </button>
        <button
          onClick={handleClose}
          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-rose-500/20 text-white/60 hover:text-rose-400 transition-colors"
          title="Close"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </header>
  );
};
