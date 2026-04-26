import data from "../../mocks/satellites.json";

export type SatelliteKind = "station" | "telescope" | "comms" | "gps" | "weather";

export interface Satellite {
  norad_id: number;
  name: string;
  kind: SatelliteKind;
  description: string;
}

export const satellites: Satellite[] = data as Satellite[];

// Placeholder orbital elements per satellite. Approximate values for the
// prototype; real numbers will come from Nyx/SGP4 in v1. Order matches
// docs/spec.md's locked panel field set: a, e, i, RAAN, ω, M, period, epoch.
export interface OrbitalElements {
  smaKm: number;
  eccentricity: number;
  inclinationDeg: number;
  raanDeg: number;
  argPerigeeDeg: number;
  meanAnomalyDeg: number;
  periodMin: number;
  epoch: string;
}

export const elementsByNorad: Record<number, OrbitalElements> = {
  25544: {
    smaKm: 6796.4,
    eccentricity: 0.000367,
    inclinationDeg: 51.64,
    raanDeg: 174.63,
    argPerigeeDeg: 117.69,
    meanAnomalyDeg: 332.43,
    periodMin: 92.93,
    epoch: "2024-01-15T11:10:36Z",
  },
  20580: {
    smaKm: 6917.0,
    eccentricity: 0.000252,
    inclinationDeg: 28.47,
    raanDeg: 60.12,
    argPerigeeDeg: 92.7,
    meanAnomalyDeg: 267.4,
    periodMin: 95.42,
    epoch: "2024-01-14T22:18:11Z",
  },
  44713: {
    smaKm: 6920.0,
    eccentricity: 0.000114,
    inclinationDeg: 53.05,
    raanDeg: 14.21,
    argPerigeeDeg: 81.3,
    meanAnomalyDeg: 278.9,
    periodMin: 95.49,
    epoch: "2024-01-15T03:42:55Z",
  },
  32260: {
    smaKm: 26561.0,
    eccentricity: 0.011,
    inclinationDeg: 55.6,
    raanDeg: 196.41,
    argPerigeeDeg: 38.7,
    meanAnomalyDeg: 322.1,
    periodMin: 717.9,
    epoch: "2024-01-15T06:04:22Z",
  },
  33591: {
    smaKm: 7224.0,
    eccentricity: 0.001461,
    inclinationDeg: 99.18,
    raanDeg: 12.85,
    argPerigeeDeg: 145.6,
    meanAnomalyDeg: 214.7,
    periodMin: 102.14,
    epoch: "2024-01-15T08:31:05Z",
  },
};
