import type { FC } from "react";
import { ConversionTask } from "../../types";
import { CheckCircle2, AlertCircle, Clock, Loader2, Folder, Trash2, ArrowRight } from "lucide-react";

interface TaskQueueProps {
  tasks: ConversionTask[];
  onClearCompleted?: () => void;
  onRevealOutput?: (path: string) => void;
}

export const TaskQueue: FC<TaskQueueProps> = ({
  tasks,
  onClearCompleted,
  onRevealOutput,
}) => {
  if (tasks.length === 0) {
    return null;
  }

  const completedCount = tasks.filter((t) => t.status === "completed").length;

  const getStatusBadge = (task: ConversionTask) => {
    switch (task.status) {
      case "completed":
        return (
          <span className="flex items-center gap-1 text-[10px] font-mono font-medium text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-2 py-0.5 rounded-full">
            <CheckCircle2 className="w-3 h-3" />
            <span>Done</span>
          </span>
        );
      case "processing":
        return (
          <span className="flex items-center gap-1 text-[10px] font-mono font-medium text-brand-light bg-brand/10 border border-brand/20 px-2 py-0.5 rounded-full">
            <Loader2 className="w-3 h-3 animate-spin" />
            <span>Transcoding</span>
          </span>
        );
      case "failed":
        return (
          <span className="flex items-center gap-1 text-[10px] font-mono font-medium text-rose-400 bg-rose-500/10 border border-rose-500/20 px-2 py-0.5 rounded-full">
            <AlertCircle className="w-3 h-3" />
            <span>Failed</span>
          </span>
        );
      case "queued":
      default:
        return (
          <span className="flex items-center gap-1 text-[10px] font-mono font-medium text-white/40 bg-white/5 border border-white/10 px-2 py-0.5 rounded-full">
            <Clock className="w-3 h-3" />
            <span>Queued</span>
          </span>
        );
    }
  };

  return (
    <div className="flex flex-col gap-2 mt-4">
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold text-white/80">Batch Queue</span>
          <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-white/5 text-white/50 border border-white/5">
            {tasks.length}
          </span>
        </div>

        {completedCount > 0 && (
          <button
            onClick={onClearCompleted}
            className="flex items-center gap-1 text-[11px] text-white/40 hover:text-white/80 transition-colors"
          >
            <Trash2 className="w-3 h-3" />
            <span>Clear Completed</span>
          </button>
        )}
      </div>

      <div className="flex flex-col gap-1.5 max-h-56 overflow-y-auto pr-1">
        {tasks.map((task) => (
          <div
            key={task.id}
            className="glass-panel rounded-lg p-2.5 flex items-center justify-between gap-3 text-xs"
          >
            <div className="flex items-center gap-2.5 min-w-0 flex-1">
              <div className="min-w-0">
                <div className="font-medium text-white/90 truncate max-w-xs">
                  {task.source_name}
                </div>
                <div className="flex items-center gap-1.5 text-[10px] font-mono text-white/40 mt-0.5">
                  <span className="uppercase font-semibold text-white/60">
                    .{task.source_format}
                  </span>
                  <ArrowRight className="w-2.5 h-2.5 text-white/30" />
                  <span className="uppercase font-bold text-brand-light">
                    .{task.target_format}
                  </span>
                  {task.error_message && (
                    <span className="text-rose-400 font-sans truncate max-w-xs ml-2">
                      ({task.error_message})
                    </span>
                  )}
                </div>
              </div>
            </div>

            <div className="flex items-center gap-2 shrink-0">
              {getStatusBadge(task)}

              {task.output_path && (
                <button
                  onClick={() => onRevealOutput?.(task.output_path!)}
                  className="w-6 h-6 flex items-center justify-center rounded hover:bg-white/10 text-white/60 hover:text-white transition-colors"
                  title="Open file location"
                >
                  <Folder className="w-3.5 h-3.5" />
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
