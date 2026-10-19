import { useRef, type MouseEvent, type FC } from "react";
import { useRadialWheel } from "../../hooks/useRadialWheel";
import { WheelItem, WheelMode } from "../../types/wheel";
import { 
  FileText, Image, Video, Music, Archive, 
  Sliders, Share2, Copy, Terminal, Maximize2,
  XCircle
} from "lucide-react";

interface RadialWheelProps {
  items: WheelItem[];
  mode: WheelMode;
  onSelect: (item: WheelItem) => void;
  onCancel?: () => void;
  sourceFileName?: string;
}

export const RadialWheel: FC<RadialWheelProps> = ({
  items,
  mode,
  onSelect,
  onCancel,
  sourceFileName,
}) => {
  const svgRef = useRef<SVGSVGElement | null>(null);

  const {
    sectors,
    hoveredIndex,
    handlePointerMove,
    handlePointerLeave,
  } = useRadialWheel({
    items,
    cx: 200,
    cy: 200,
    innerRadius: 68,
    outerRadius: 154,
    hoverRadiusDelta: 16,
  });

  const activeSector = hoveredIndex !== null ? sectors[hoveredIndex] : null;

  const onMouseMove = (e: MouseEvent<SVGSVGElement>) => {
    if (!svgRef.current) return;
    const rect = svgRef.current.getBoundingClientRect();
    handlePointerMove(e.clientX, e.clientY, rect);
  };

  const getIcon = (iconName?: string, category?: string) => {
    switch (iconName || category) {
      case "Image":
      case "image":
        return <Image className="w-4 h-4" />;
      case "Video":
      case "video":
      case "Film":
        return <Video className="w-4 h-4" />;
      case "Audio":
      case "audio":
      case "Scissors":
        return <Music className="w-4 h-4" />;
      case "Archive":
      case "archive":
        return <Archive className="w-4 h-4" />;
      case "Share2":
        return <Share2 className="w-4 h-4" />;
      case "Copy":
        return <Copy className="w-4 h-4" />;
      case "Terminal":
        return <Terminal className="w-4 h-4" />;
      case "Maximize2":
        return <Maximize2 className="w-4 h-4" />;
      case "Sliders":
        return <Sliders className="w-4 h-4" />;
      default:
        return <FileText className="w-4 h-4" />;
    }
  };

  const getModeColor = () => {
    switch (mode) {
      case "format":
        return "text-brand-light";
      case "tool":
        return "text-amber-300";
      case "action":
        return "text-emerald-300";
    }
  };

  const getModeTitle = () => {
    switch (mode) {
      case "format":
        return "Target Formats";
      case "tool":
        return "Advanced Tools";
      case "action":
        return "Quick Actions";
    }
  };

  return (
    <div className="flex flex-col items-center justify-center p-4">
      {/* HUD Header */}
      <div className="mb-2 text-center pointer-events-none">
        <div className={`text-xs uppercase tracking-wider font-mono font-semibold ${getModeColor()}`}>
          {getModeTitle()}
        </div>
        {sourceFileName && (
          <div className="text-[11px] text-white/50 truncate max-w-xs mt-0.5">
            {sourceFileName}
          </div>
        )}
      </div>

      {/* SVG Radial Wheel Canvas */}
      <div className="relative w-[360px] h-[360px] flex items-center justify-center drop-shadow-2xl">
        <svg
          ref={svgRef}
          viewBox="0 0 400 400"
          className="w-full h-full cursor-pointer select-none"
          onMouseMove={onMouseMove}
          onMouseLeave={handlePointerLeave}
          onClick={() => {
            if (activeSector) {
              onSelect(activeSector.item);
            } else {
              onCancel?.();
            }
          }}
        >
          <defs>
            <filter id="glow-format" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="6" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
            <filter id="glow-tool" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="6" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
            <filter id="glow-action" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="6" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
          </defs>

          {/* Slices */}
          {sectors.map((sec) => {
            const isHovered = sec.isHovered;
            let fillColor = "rgba(21, 23, 26, 0.75)";
            let strokeColor = "rgba(255, 255, 255, 0.10)";
            let strokeWidth = 1.2;
            let filter: string | undefined = undefined;

            if (isHovered) {
              if (mode === "format") {
                fillColor = "rgba(94, 106, 210, 0.40)";
                strokeColor = "rgba(165, 180, 252, 0.70)";
                filter = "url(#glow-format)";
              } else if (mode === "tool") {
                fillColor = "rgba(245, 158, 11, 0.35)";
                strokeColor = "rgba(252, 211, 77, 0.70)";
                filter = "url(#glow-tool)";
              } else {
                fillColor = "rgba(16, 185, 129, 0.35)";
                strokeColor = "rgba(110, 231, 183, 0.70)";
                filter = "url(#glow-action)";
              }
              strokeWidth = 2.0;
            }

            return (
              <g key={sec.index} className="transition-all duration-200">
                <path
                  d={sec.pathData}
                  fill={fillColor}
                  stroke={strokeColor}
                  strokeWidth={strokeWidth}
                  filter={filter}
                  className="transition-all duration-150 ease-out"
                />

                {/* Slice Label */}
                <text
                  x={sec.labelPosition.x}
                  y={sec.labelPosition.y}
                  textAnchor="middle"
                  dominantBaseline="central"
                  fill={isHovered ? "#ffffff" : "rgba(255, 255, 255, 0.75)"}
                  className="text-[11px] font-semibold font-mono uppercase tracking-wider pointer-events-none transition-all"
                >
                  {sec.item.label}
                </text>
              </g>
            );
          })}

          {/* Central Deadzone / Cancel Disc */}
          <circle
            cx="200"
            cy="200"
            r="62"
            fill="rgba(15, 16, 18, 0.90)"
            stroke={hoveredIndex === null ? "rgba(255, 255, 255, 0.18)" : "rgba(255, 255, 255, 0.08)"}
            strokeWidth="1.5"
            className="transition-colors backdrop-blur-xl"
          />
        </svg>

        {/* Center Content Overlay */}
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
          {activeSector ? (
            <div className="text-center px-4 animate-in fade-in zoom-in-95 duration-100">
              <div className="flex justify-center mb-1">
                {getIcon(activeSector.item.icon, activeSector.item.category)}
              </div>
              <div className="text-xs font-bold text-white font-mono uppercase tracking-wide truncate max-w-[90px]">
                {activeSector.item.label}
              </div>
              <div className="text-[9px] text-white/50 tracking-tight mt-0.5">
                Click to apply
              </div>
            </div>
          ) : (
            <div className="text-center px-4">
              <XCircle className="w-5 h-5 text-white/30 mx-auto mb-1" />
              <div className="text-[10px] font-medium text-white/40 uppercase tracking-widest">
                Release
              </div>
              <div className="text-[9px] text-white/30 mt-0.5">
                or click center
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Keyboard Hint */}
      <div className="mt-3 flex items-center gap-3 text-[11px] text-white/40">
        <span className="flex items-center gap-1">
          <kbd className="px-1 py-0.2 rounded bg-white/5 border border-white/10 text-white/60 font-mono text-[9px]">
            Shift
          </kbd>
          <span>Formats</span>
        </span>
        <span className="text-white/20">•</span>
        <span className="flex items-center gap-1">
          <kbd className="px-1 py-0.2 rounded bg-white/5 border border-white/10 text-white/60 font-mono text-[9px]">
            Alt
          </kbd>
          <span>Tools</span>
        </span>
        <span className="text-white/20">•</span>
        <span className="flex items-center gap-1">
          <kbd className="px-1 py-0.2 rounded bg-white/5 border border-white/10 text-white/60 font-mono text-[9px]">
            Ctrl
          </kbd>
          <span>Actions</span>
        </span>
      </div>
    </div>
  );
};
