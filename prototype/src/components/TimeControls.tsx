import { Pause, Play, RotateCcw } from "lucide-react";
import { cn } from "@/lib/cn";
import type { Speed } from "@/lib/simClock";

const SPEEDS: Speed[] = [1, 10, 100, 1000];

export interface TimeControlsProps {
  simTimeMs: number;
  speed: Speed;
  playing: boolean;
  onSpeedChange: (s: Speed) => void;
  onTogglePlay: () => void;
  onNow: () => void;
  className?: string;
}

export function TimeControls({
  simTimeMs,
  speed,
  playing,
  onSpeedChange,
  onTogglePlay,
  onNow,
  className,
}: TimeControlsProps) {
  const stamp = new Date(simTimeMs).toISOString().replace("T", " ").slice(0, 19) + " UTC";

  return (
    <div
      className={cn(
        "flex items-center gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2",
        className,
      )}
    >
      <button
        type="button"
        onClick={onTogglePlay}
        aria-label={playing ? "Pause" : "Play"}
        className="flex h-9 w-9 items-center justify-center rounded-md bg-[var(--color-surface-elevated)] text-[var(--color-fg)] hover:bg-[color-mix(in_oklch,var(--color-surface-elevated),white_8%)]"
      >
        {playing ? <Pause size={16} /> : <Play size={16} />}
      </button>

      <div className="flex items-center gap-1 rounded-md bg-[var(--color-surface-elevated)] p-1">
        {SPEEDS.map((s) => (
          <button
            key={s}
            type="button"
            onClick={() => onSpeedChange(s)}
            aria-pressed={s === speed}
            className={cn(
              "rounded px-2.5 py-1 font-mono text-xs tabular-nums transition-colors",
              s === speed
                ? "bg-[var(--color-accent)] text-[var(--color-bg)]"
                : "text-[var(--color-muted)] hover:text-[var(--color-fg)]",
            )}
          >
            {s}×
          </button>
        ))}
      </div>

      <button
        type="button"
        onClick={onNow}
        className="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs text-[var(--color-muted)] hover:text-[var(--color-fg)]"
      >
        <RotateCcw size={14} />
        Now
      </button>

      <div className="ml-auto font-mono text-xs tabular-nums text-[var(--color-muted)]">
        {stamp}
      </div>
    </div>
  );
}
