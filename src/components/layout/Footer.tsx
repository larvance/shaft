import type { FC } from "react";
import { WheelMode } from "../../types/wheel";
import { Cpu, ShieldCheck } from "lucide-react";

interface FooterProps {
  activeMode: WheelMode | null;
  totalConversions?: number;
}

export const Footer: FC<FooterProps> = ({
  activeMode,
  totalConversions = 275,
}) => {
  return (
    <footer className="h-8 flex items-center justify-between px-3 text-[11px] font-mono text-white/40 bg-obsidian-950/40 border-t border-hairline backdrop-blur-md select-none shrink-0 z-40">
      <div className="flex items-center gap-2">
        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
        <span className="flex items-center gap-1 text-white/60">
          <Cpu className="w-3 h-3 text-brand-light" />
          <span>Rust Transcoder</span>
        </span>
        <span className="text-white/20">•</span>
        <span className="text-white/50">{totalConversions} Formats Available</span>
      </div>

      <div className="flex items-center gap-3">
        {activeMode ? (
          <span className="px-1.5 py-0.5 rounded bg-brand/20 border border-brand/40 text-brand-light font-sans text-[10px] font-semibold animate-in fade-in duration-100">
            {activeMode === "format"
              ? "Format Wheel (Shift)"
              : activeMode === "tool"
              ? "Tools Wheel (Alt)"
              : "Actions Wheel (Ctrl)"}
          </span>
        ) : (
          <span className="text-white/30 text-[10px]">
            Hold Shift, Alt, or Ctrl for Radial Wheels
          </span>
        )}

        <div className="flex items-center gap-1 text-white/40 border-l border-white/10 pl-2">
          <ShieldCheck className="w-3 h-3 text-emerald-400" />
          <span className="text-[10px]">Pure Safe Rust</span>
        </div>
      </div>
    </footer>
  );
};
