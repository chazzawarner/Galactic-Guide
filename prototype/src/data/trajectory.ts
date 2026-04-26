import raw from "../../mocks/25544-trajectory.json";
import type { Sample } from "@/lib/interpolate";

export interface TrajectoryWindow {
  norad_id: number;
  name: string;
  frame: "eci_j2000";
  start_at: string;
  duration_s: number;
  step_s: number;
  include_velocity: boolean;
  samples: Sample[];
}

export const issTrajectory: TrajectoryWindow = raw as TrajectoryWindow;
