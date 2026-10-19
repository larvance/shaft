import type { FC } from "react";
import { ConversionTask } from "../../types";
import { 
  CheckCircle2, AlertCircle, Clock, Loader2, Folder, 
  Trash2, ArrowRight, XCircle, Ban
} from "lucide-react";

interface TaskQueueProps {
  tasks: ConversionTask[];
  onClearCompleted?: () => void;
  onCancelTask?: (taskId: string) => void;
  onCancelAll?: () => void;
  onRevealOutput?: (path: string) => void;
}

export const TaskQueue: FC<TaskQueueProps> = ({
  tasks,
  onClearCompleted,
  onCancelTask,
  onCancelAll,
  onRevealOutput,
}) => {
  if (tasks.length === 0) {
    return null;
  }

  const runningCount = tasks.filter((t) => t.status === "processing" || t.status === "queued").length;
  const finishedCount = tasks.filter((t) => t.status === "completed" || t.status === "cancelled" || t.status === "failed").length;

  const formatSpeed = (bytesPerSec?: number) => {
    if (!bytesPerSec || bytesPerSec === 0) return null;
    const mb = bytesPerSec / (1024 * 1024);
    return `${mb.toFixed(1)} MB/s`;
  };

  const formatEta = (seconds?: number) => {
    if (!seconds || seconds <= 0) return null;
    return `ETA: ${seconds.toFixed(1)}s`;
  };

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
            <span>{Math.round(task.progress * 100)}%</span>
          </span>
        );
      case "failed":
        return (
          <span className="flex items-center gap-1 text-[10px] font-mono font-medium text-rose-400 bg-rose-500/10 border border-rose-500/20 px-2 py-0.5 rounded-full">
            <AlertCircle className="w-3 h-3" />
            <span>Failed</span>
          </span>
        );
      case "cancelled":
        return (
          <span className="flex items-center gap-1 text-[10px] font-mono font-medium text-amber-400 bg-amber-500/10 border border-amber-500/20 px-2 py-0.5 rounded-full">
            <Ban className="w-3 h-3" />
            <span>Cancelled</span>
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
      {/* Queue Header & Global Actions */}
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-2">
          <span className="text-xs font-semibold text-white/90">Transcoding Queue</span>
          <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-white/5 text-white/50 border border-white/10">
            {tasks.length}
          </span>
        </div>

        <div className="flex items-center gap-2">
          {runningCount > 0 && onCancelAll && (
            <button
              onClick={onCancelAll}
              className="flex items-center gap-1 text-[11px] text-rose-400/80 hover:text-rose-400 px-2 py-0.5 rounded hover:bg-rose-500/10 transition-colors"
            >
              <XCircle className="w-3 h-3" />
              <span>Cancel All</span>
            </button>
          )}

          {finishedCount > 0 && onClearCompleted && (
            <button
              onClick={onClearCompleted}
              className="flex items-center gap-1 text-[11px] text-white/40 hover:text-white/80 px-2 py-0.5 rounded hover:bg-white/5 transition-colors"
            >
              <Trash2 className="w-3 h-3" />
              <span>Clear Finished</span>
            </button>
          )}
        </div>
      </div>

      {/* Task List */}
      <div className="flex flex-col gap-2 max-h-64 overflow-y-auto pr-1">
        {tasks.map((task) => (
          <div
            key={task.id}
            className="glass-panel rounded-xl p-3 flex flex-col gap-2 text-xs transition-all hover:border-white/15"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0 flex-1">
                <div className="font-medium text-white/95 truncate max-w-sm">
                  {task.source_name}
                </div>
                <div className="flex items-center gap-2 text-[10px] font-mono text-white/40 mt-0.5">
                  <span className="uppercase font-semibold text-white/70">
                    .{task.source_format}
                  </span>
                  <ArrowRight className="w-2.5 h-2.5 text-white/30" />
                  <span className="uppercase font-bold text-brand-light">
                    .{task.target_format}
                  </span>

                  {task.speed_bytes_per_sec && (
                    <>
                      <span>•</span>
                      <span className="text-white/60">{formatSpeed(task.speed_bytes_per_sec)}</span>
                    </>
                  )}

                  {task.eta_seconds && (
                    <>
                      <span>•</span>
                      <span className="text-white/60">{formatEta(task.eta_seconds)}</span>
                    </>
                  )}

                  {task.error_message && (
                    <span className="text-rose-400 font-sans truncate max-w-xs ml-1">
                      ({task.error_message})
                    </span>
                  )}
                </div>
              </div>

              {/* Status and Action Buttons */}
              <div className="flex items-center gap-2 shrink-0">
                {getStatusBadge(task)}

                {task.status === "processing" && onCancelTask && (
                  <button
                    onClick={() => onCancelTask(task.id)}
                    className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-rose-500/20 text-white/40 hover:text-rose-400 transition-colors"
                    title="Cancel conversion"
                  >
                    <XCircle className="w-3.5 h-3.5" />
                  </button>
                )}

                {task.output_path && (
                  <button
                    onClick={() => onRevealOutput?.(task.output_path!)}
                    className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-white/10 text-white/60 hover:text-white transition-colors"
                    title="Reveal in file manager"
                  >
                    <Folder className="w-3.5 h-3.5" />
                  </button>
                )}
              </div>
            </div>

            {/* Granular Progress Bar */}
            {task.status === "processing" && (
              <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden">
                <div
                  className="bg-brand-light h-full rounded-full transition-all duration-150 ease-out shadow-[0_0_8px_rgba(129,140,248,0.5)]"
                  style={{ width: `${Math.max(5, task.progress * 100)}%` }}
                />
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
