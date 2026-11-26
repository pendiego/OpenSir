import type { DeadZone, Linear } from "./model";

/**
 * Curve points on the wire (docs/protocol/cyclone2-profile.md §8): five (original, target) byte
 * pairs; pair 0 is (0,0) and pair 4 is (255,255). Pairs 1-3 are the user's points, scaled to
 * 0-255 inside the deadzone (front..back) and anti-deadzone (antiFront..antiBack) brackets.
 * The UI works in user units (0-100 of the full input/output range).
 */
const toUser = (e: number, lo: number, hi: number) => Math.round((e / 255) * (hi - lo) + lo);

function toWire(v: number, lo: number, hi: number): number {
  if (hi <= lo) return 0;
  return Math.min(255, Math.max(0, Math.round(((v - lo) / (hi - lo)) * 255)));
}

/** The three user points in 0-100 units. */
export function userPoints(dead: DeadZone, linear: Linear): [number, number][] {
  return [1, 2, 3].map((i) => [
    toUser(linear.points[i][0], dead.front, dead.back),
    toUser(linear.points[i][1], dead.antiFront, dead.antiBack),
  ]);
}

/**
 * Sets user point `k` (0-2) from 0-100 units. An axis whose displayed value did not change keeps
 * its wire byte: decode/encode rounding does not round-trip (front 10, back 100: byte 42 shows
 * as 25, and 25 encodes to 43), so a Y-only nudge must not rewrite X.
 */
export function setUserPoint(dead: DeadZone, linear: Linear, k: number, x: number, y: number) {
  const [ox, oy] = linear.points[k + 1];
  linear.points[k + 1] = [
    toUser(ox, dead.front, dead.back) === x ? ox : toWire(x, dead.front, dead.back),
    toUser(oy, dead.antiFront, dead.antiBack) === y ? oy : toWire(y, dead.antiFront, dead.antiBack),
  ];
}

/** Polyline through the whole curve, deadzone ends included, in 0-100 units. */
export function curvePath(dead: DeadZone, linear: Linear): [number, number][] {
  return [[dead.front, dead.antiFront], ...userPoints(dead, linear), [dead.back, dead.antiBack]];
}
