import { cn } from "@/lib/cn";
import type { Satellite, SatelliteKind, OrbitalElements } from "@/data/satellites";

const KIND_LABEL: Record<SatelliteKind, string> = {
  station: "Station",
  telescope: "Telescope",
  comms: "Comms",
  gps: "GPS",
  weather: "Weather",
};

export interface SatellitePanelProps {
  satellite: Satellite;
  elements: OrbitalElements;
  className?: string;
}

export function SatellitePanel({ satellite, elements, className }: SatellitePanelProps) {
  const rows: Array<[string, string]> = [
    ["a", `${elements.smaKm.toFixed(1)} km`],
    ["e", elements.eccentricity.toFixed(6)],
    ["i", `${elements.inclinationDeg.toFixed(2)}°`],
    ["RAAN", `${elements.raanDeg.toFixed(2)}°`],
    ["ω", `${elements.argPerigeeDeg.toFixed(2)}°`],
    ["M", `${elements.meanAnomalyDeg.toFixed(2)}°`],
    ["period", `${elements.periodMin.toFixed(2)} min`],
    ["epoch", elements.epoch],
  ];

  return (
    <section
      className={cn(
        "flex w-[320px] flex-col gap-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5",
        className,
      )}
    >
      <header className="flex flex-col gap-1.5">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-medium text-[var(--color-fg)]">{satellite.name}</h2>
          <span className="rounded-full bg-[var(--color-surface-elevated)] px-2 py-0.5 text-[10px] uppercase tracking-wider text-[var(--color-muted)]">
            {KIND_LABEL[satellite.kind]}
          </span>
        </div>
        <p className="font-mono text-xs text-[var(--color-muted)]">NORAD {satellite.norad_id}</p>
        <p className="text-xs leading-relaxed text-[var(--color-muted)]">{satellite.description}</p>
      </header>

      <div className="h-px bg-[var(--color-border)]" />

      <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-2 font-mono text-xs">
        {rows.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-[var(--color-muted)]">{label}</dt>
            <dd className="tabular-nums text-right text-[var(--color-fg)]">{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
