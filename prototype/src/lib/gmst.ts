// IAU 1982 Greenwich Mean Sidereal Time — radians.
// Used as a Y-axis rotation on the Earth mesh while satellites stay in ECI.

const SECONDS_PER_DAY = 86400;
const J2000_UNIX_S = 946727935.816; // 2000-01-01T12:00:00 TT in unix seconds

export function gmstFromUnix(unixSeconds: number): number {
  // Centuries since J2000 (TT ≈ UT1 to the precision a feel-prototype cares about).
  const T = (unixSeconds - J2000_UNIX_S) / (SECONDS_PER_DAY * 36525);

  // Polynomial in seconds of time, IAU 1982.
  const gmstSeconds =
    67310.54841 +
    (876600 * 3600 + 8640184.812866) * T +
    0.093104 * T * T -
    6.2e-6 * T * T * T;

  // Convert seconds-of-time to radians, modulo 2π.
  const radians = ((gmstSeconds % SECONDS_PER_DAY) / SECONDS_PER_DAY) * 2 * Math.PI;
  return ((radians % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
}
