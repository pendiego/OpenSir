import type { Frame, LightAnimation, Rgb } from "./model";
import type { Key } from "./i18n";

/** One key frame by LED: grips (lower left/right), the Home ring, the back light (upper right) and the unused upper-left slot. */
interface Step {
  ll: string;
  lr: string;
  home: string;
  ur: string;
  /** RGB2, which no UI shows; mirrors `ur` when absent. */
  ul?: string;
}

export interface LightPreset {
  id: string;
  name: Key;
  /** 0..20, higher is faster (stored on the device as 20 - speed). */
  speed: number;
  steps: Step[];
}

const all = (c: string): Step => ({ ll: c, lr: c, home: c, ur: c });
const OFF = "#000000";

// The first six presets match the device's stock animations, so they look familiar; the rest are ours.
// At most 8 key frames (the device limit).
export const LIGHT_PRESETS: LightPreset[] = [
  {
    id: "flow",
    name: "presetFlow",
    speed: 17,
    steps: [
      { ll: "#00ff00", lr: "#ffff00", home: "#ff0000", ur: "#ff0000", ul: "#0000ff" },
      { ll: "#0000ff", lr: "#00ff80", home: "#ff8000", ur: "#ffff00", ul: "#ff0000" },
      { ll: "#800080", lr: "#0000ff", home: "#ffff00", ur: "#00ff00", ul: "#ff80ff" },
      { ll: "#ff0000", lr: "#ff0000", home: "#14ff01", ur: "#0000ff", ul: "#808000" },
      { ll: "#ff0000", lr: "#808000", home: "#000fff", ur: "#ff0080", ul: "#008000" },
    ],
  },
  {
    id: "rainbow",
    name: "presetRainbow",
    speed: 10,
    steps: ["#ff0000", "#ffa500", "#ffff00", "#00ff00", "#00ffff", "#0000ff", "#ff00ff", "#ff0000"].map(all),
  },
  {
    id: "pulse",
    name: "presetPulse",
    speed: 5,
    steps: [
      { ll: "#8000ff", lr: "#8000ff", home: "#ff00c8", ur: "#ff0000", ul: "#ff0000" },
      { ll: "#0080ff", lr: "#0080ff", home: "#0080ff", ur: OFF, ul: OFF },
    ],
  },
  { id: "standoff", name: "presetStandoff", speed: 10, steps: [{ ll: "#0040ff", lr: "#ff0500", home: "#ffffff", ur: "#ff0500", ul: "#00b3ff" }] },
  { id: "alarm", name: "presetAlarm", speed: 18, steps: [all("#ff0000"), all(OFF), all("#ff0000"), all(OFF), all(OFF), all(OFF)] },
  { id: "off", name: "presetOff", speed: 10, steps: [all(OFF)] },
  { id: "white", name: "presetWhite", speed: 10, steps: [all("#ffffff")] },
  { id: "amber", name: "presetAmber", speed: 10, steps: [all("#ffa31a")] },
  { id: "breathe", name: "presetBreathe", speed: 6, steps: [all("#ffffff"), all("#3a3a3a"), all(OFF), all("#3a3a3a")] },
  {
    id: "spectrum",
    name: "presetSpectrum",
    speed: 12,
    steps: [
      { ll: "#ff0000", lr: "#00ff00", home: "#0040ff", ur: "#ff00c0" },
      { ll: "#ff00c0", lr: "#ff0000", home: "#00ff00", ur: "#0040ff" },
      { ll: "#0040ff", lr: "#ff00c0", home: "#ff0000", ur: "#00ff00" },
      { ll: "#00ff00", lr: "#0040ff", home: "#ff00c0", ur: "#ff0000" },
    ],
  },
  {
    id: "police",
    name: "presetPolice",
    speed: 18,
    steps: [
      { ll: "#ff0000", lr: OFF, home: "#ff0000", ur: OFF },
      { ll: OFF, lr: "#0030ff", home: "#0030ff", ur: "#0030ff" },
    ],
  },
  {
    id: "fire",
    name: "presetFire",
    speed: 15,
    steps: [
      { ll: "#ff2000", lr: "#ff6000", home: "#ff4000", ur: "#ff1000" },
      { ll: "#ff6000", lr: "#ff3000", home: "#ffa000", ur: "#ff4000" },
      { ll: "#ff4000", lr: "#ff1000", home: "#ff5000", ur: "#ff8000" },
      { ll: "#ff1000", lr: "#ff7000", home: "#ff3000", ur: "#ff2000" },
    ],
  },
  {
    id: "ocean",
    name: "presetOcean",
    speed: 7,
    steps: [
      { ll: "#0040ff", lr: "#00c0ff", home: "#0080ff", ur: "#00ffd0" },
      { ll: "#00c0ff", lr: "#00ffd0", home: "#0040ff", ur: "#0080ff" },
      { ll: "#00ffd0", lr: "#0080ff", home: "#00c0ff", ur: "#0040ff" },
    ],
  },
  {
    id: "aurora",
    name: "presetAurora",
    speed: 5,
    steps: [
      { ll: "#00ff80", lr: "#8000ff", home: "#00ffc0", ur: "#4000ff" },
      { ll: "#4000ff", lr: "#00ff80", home: "#8000ff", ur: "#00ffc0" },
      { ll: "#8000ff", lr: "#00ffc0", home: "#00ff80", ur: "#4000ff" },
    ],
  },
  {
    id: "sunset",
    name: "presetSunset",
    speed: 5,
    steps: [
      { ll: "#ff6000", lr: "#ff0060", home: "#ffa000", ur: "#8000ff" },
      { ll: "#ff0060", lr: "#8000ff", home: "#ff6000", ur: "#ffa000" },
      { ll: "#8000ff", lr: "#ffa000", home: "#ff0060", ur: "#ff6000" },
    ],
  },
  {
    id: "neon",
    name: "presetNeon",
    speed: 14,
    steps: [
      { ll: "#ff00ff", lr: "#00ffff", home: "#ff00ff", ur: "#00ffff" },
      { ll: "#00ffff", lr: "#ff00ff", home: "#00ffff", ur: "#ff00ff" },
    ],
  },
  { id: "strobe", name: "presetStrobe", speed: 20, steps: [all("#ffffff"), all(OFF)] },
];

const rgb = (hex: string): Rgb => [parseInt(hex.slice(1, 3), 16), parseInt(hex.slice(3, 5), 16), parseInt(hex.slice(5, 7), 16)];

/** Device frame order RGB0..RGB4: lower left, lower right, upper left, Home, upper right. */
export function stepFrame(s: Step): Frame {
  return [rgb(s.ll), rgb(s.lr), rgb(s.ul ?? s.ur), rgb(s.home), rgb(s.ur)];
}

/** Writes a preset into an animation: speed, key frame count, and all 8 frames (black past the preset). */
export function applyPreset(anim: LightAnimation, p: LightPreset): void {
  anim.speed = p.speed;
  anim.keyFrameCount = p.steps.length;
  anim.frames = Array.from({ length: 8 }, (_, i) => stepFrame(p.steps[i] ?? all(OFF)));
}

/** The preset this animation currently holds, if it is unchanged since it was applied. */
export function matchPreset(anim: LightAnimation): string | null {
  const same = (a: Frame, b: Frame) => a.every((c, i) => c.every((v, j) => v === b[i][j]));
  const p = LIGHT_PRESETS.find(
    (p) => p.speed === anim.speed && p.steps.length === anim.keyFrameCount && p.steps.every((s, i) => same(stepFrame(s), anim.frames[i])),
  );
  return p?.id ?? null;
}
