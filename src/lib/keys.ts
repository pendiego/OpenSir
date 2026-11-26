import type { Key } from "./i18n";
import type { Button, FunctionKey, Profile, TouchKey, Trigger } from "./model";

export type KeyTarget =
  | { kind: "button"; index: number }
  | { kind: "fn"; name: "fl1" | "fr1" }
  | { kind: "trigger"; name: "l2" | "r2" }
  | { kind: "extra"; index: number }
  /** T3CE touch strip keys Ext8, Ext9. */
  | { kind: "touch"; index: number };

export type StickAxis = "lx" | "ly" | "rx" | "ry";

/** Hotspot outline on a cover: circle, or rect with optional clockwise rotation in degrees. */
export type KeyShape = { kind: "circle"; r: number } | { kind: "rect"; w: number; h: number; rx: number; rot?: number };

export interface KeyInfo {
  /** Key code name from the device (`cyclone2-keycodes.json`), also used by live button names. */
  id: string;
  label: string;
  /** Localized name for keys whose label is an abbreviation or device code. */
  name?: Key;
  target: KeyTarget;
  /** Stick caps: the live view moves the cap by these two axes. */
  axes?: readonly [StickAxis, StickAxis];
}

const btn = (index: number): KeyTarget => ({ kind: "button", index });

/** The 20 keys drawn on the controller. Every one is mappable. Positions live in src/lib/pads/<id>.json. */
export const KEYS: KeyInfo[] = [
  { id: "L2", label: "LT", name: "spotLT", target: { kind: "trigger", name: "l2" } },
  { id: "L1", label: "LB", name: "spotLB", target: btn(4) },
  { id: "L3", label: "LS", name: "spotLS", target: btn(6), axes: ["lx", "ly"] },
  { id: "Up", label: "Up", name: "spotDUp", target: btn(0) },
  { id: "Under", label: "Down", name: "spotDDown", target: btn(1) },
  { id: "Left", label: "Left", name: "spotDLeft", target: btn(2) },
  { id: "Right", label: "Right", name: "spotDRight", target: btn(3) },
  { id: "TRI", label: "Y", target: btn(11) },
  { id: "SQU", label: "X", target: btn(10) },
  { id: "CIR", label: "B", target: btn(9) },
  { id: "CRO", label: "A", target: btn(8) },
  { id: "R1", label: "RB", name: "spotRB", target: btn(5) },
  { id: "R2", label: "RT", name: "spotRT", target: { kind: "trigger", name: "r2" } },
  { id: "R3", label: "RS", name: "spotRS", target: btn(7), axes: ["rx", "ry"] },
  { id: "PS", label: "Home", name: "spotHome", target: btn(12) },
  { id: "Sel", label: "View", name: "spotView", target: btn(13) },
  { id: "Sta", label: "Menu", name: "spotMenu", target: btn(14) },
  { id: "Camera", label: "Share", name: "spotShare", target: btn(15) },
  { id: "FL1", label: "FL1", name: "spotPaddleL", target: { kind: "fn", name: "fl1" } },
  { id: "FR1", label: "FR1", name: "spotPaddleR", target: { kind: "fn", name: "fr1" } },
];

export type KeyData = Button | Trigger | FunctionKey | TouchKey;

export function keyOf(p: Profile, t: KeyTarget): KeyData {
  if (t.kind === "button") return p.buttons[t.index];
  if (t.kind === "extra") return p.extra[t.index];
  if (t.kind === "touch") return p.touch[t.index];
  return p[t.name];
}

/** A key is marked on the drawing when it is mapped or has an enabled macro. */
export function marked(p: Profile, t: KeyTarget): boolean {
  const k = keyOf(p, t);
  return k.mapped || ("macroEnabled" in k && k.macroEnabled);
}

export function sameTarget(a: KeyTarget, b: KeyTarget | null): boolean {
  if (!b || a.kind !== b.kind) return false;
  if (a.kind === "button" && b.kind === "button") return a.index === b.index;
  if (a.kind === "fn" && b.kind === "fn") return a.name === b.name;
  if (a.kind === "trigger" && b.kind === "trigger") return a.name === b.name;
  if (a.kind === "extra" && b.kind === "extra") return a.index === b.index;
  if (a.kind === "touch" && b.kind === "touch") return a.index === b.index;
  return false;
}
