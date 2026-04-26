import { cn } from "@/lib/cn";
import { Globe } from "./Globe";
import { SatellitePanel } from "./SatellitePanel";
import { TimeControls } from "./TimeControls";
import { useSimClock } from "@/lib/simClock";
import type { Satellite } from "@/data/satellites";
import { elementsByNorad } from "@/data/satellites";
import type { TrajectoryWindow } from "@/data/trajectory";

export type DashboardLayout = "sidebar" | "drawer" | "overlay" | "bottom-sheet";

export interface DashboardProps {
  satellites: Satellite[];
  selected: Satellite;
  trajectory: TrajectoryWindow;
  layout: DashboardLayout;
}

export function Dashboard({ selected, trajectory, layout }: DashboardProps) {
  const clock = useSimClock(Date.parse(trajectory.start_at));
  const elements = elementsByNorad[selected.norad_id];

  const globe = <Globe trajectory={trajectory} simTimeMs={clock.simTimeMs} />;

  const panel = (
    <SatellitePanel satellite={selected} elements={elements} className="w-full max-w-[320px]" />
  );

  const controls = (
    <TimeControls
      simTimeMs={clock.simTimeMs}
      speed={clock.speed}
      playing={clock.playing}
      onSpeedChange={clock.setSpeed}
      onTogglePlay={clock.toggle}
      onNow={clock.now}
    />
  );

  if (layout === "sidebar") {
    return (
      <div className="flex h-full w-full bg-[var(--color-bg)]">
        <aside className="flex w-[360px] flex-col gap-4 border-r border-[var(--color-border)] p-4">
          {panel}
        </aside>
        <main className="flex flex-1 flex-col">
          <div className="flex-1">{globe}</div>
          <div className="border-t border-[var(--color-border)] p-3">{controls}</div>
        </main>
      </div>
    );
  }

  if (layout === "drawer") {
    return (
      <div className="relative h-full w-full bg-[var(--color-bg)]">
        <div className="absolute inset-0">{globe}</div>
        <aside className="absolute top-4 left-4 bottom-20">{panel}</aside>
        <div className="absolute right-4 left-4 bottom-4">{controls}</div>
      </div>
    );
  }

  if (layout === "overlay") {
    return (
      <div className="relative h-full w-full bg-[var(--color-bg)]">
        <div className="absolute inset-0">{globe}</div>
        <aside
          className={cn(
            "absolute top-4 right-4 backdrop-blur-md",
            "[&>section]:bg-[color-mix(in_oklch,var(--color-surface),transparent_25%)]",
          )}
        >
          {panel}
        </aside>
        <div className="absolute right-4 bottom-4 left-4 mx-auto max-w-[640px] backdrop-blur-md">
          {controls}
        </div>
      </div>
    );
  }

  // bottom-sheet
  return (
    <div className="relative flex h-full w-full flex-col bg-[var(--color-bg)]">
      <div className="flex-1">{globe}</div>
      <div className="border-t border-[var(--color-border)] p-3">{controls}</div>
      <div className="border-t border-[var(--color-border)] p-4">
        <div className="mx-auto">{panel}</div>
      </div>
    </div>
  );
}
