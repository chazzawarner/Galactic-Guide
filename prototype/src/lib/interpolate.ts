// Cubic Hermite interpolation between sampled trajectory windows.
// Per docs/architecture.md § Time-controlled propagation:
// 10 s steps at LEO ≈ 75 km of motion — position-only interpolation drifts
// noticeably; using returned velocities gives C1 continuity that reads as smooth.

export type Vec3 = [number, number, number];

export interface Sample {
  t: number; // seconds from window start
  r_km: Vec3;
  v_km_s: Vec3;
}

function hermite(p0: number, m0: number, p1: number, m1: number, dt: number, s: number) {
  const s2 = s * s;
  const s3 = s2 * s;
  const h00 = 2 * s3 - 3 * s2 + 1;
  const h10 = s3 - 2 * s2 + s;
  const h01 = -2 * s3 + 3 * s2;
  const h11 = s3 - s2;
  return h00 * p0 + h10 * dt * m0 + h01 * p1 + h11 * dt * m1;
}

// `t` is seconds from window start. Clamps to the window's first/last sample.
export function interpolateSamples(samples: Sample[], t: number): Vec3 {
  if (samples.length === 0) return [0, 0, 0];
  if (t <= samples[0].t) return samples[0].r_km;
  if (t >= samples[samples.length - 1].t) return samples[samples.length - 1].r_km;

  let lo = 0;
  let hi = samples.length - 1;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if (samples[mid].t <= t) lo = mid;
    else hi = mid;
  }

  const a = samples[lo];
  const b = samples[hi];
  const dt = b.t - a.t;
  const s = (t - a.t) / dt;

  return [
    hermite(a.r_km[0], a.v_km_s[0], b.r_km[0], b.v_km_s[0], dt, s),
    hermite(a.r_km[1], a.v_km_s[1], b.r_km[1], b.v_km_s[1], dt, s),
    hermite(a.r_km[2], a.v_km_s[2], b.r_km[2], b.v_km_s[2], dt, s),
  ];
}
