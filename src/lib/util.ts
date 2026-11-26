import type { Rgb } from "./model";

export function clamp(n: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, n));
}

export function hexId(n: number): string {
  return "0x" + n.toString(16).toUpperCase().padStart(4, "0");
}

export function hexRgb(c: Rgb): string {
  return "#" + c.map((v) => v.toString(16).padStart(2, "0")).join("");
}

export function parseRgb(h: string): Rgb {
  return [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
}
