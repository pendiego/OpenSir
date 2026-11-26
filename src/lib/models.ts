// UI view of the GameSir models in crates/opensir-core/src/models.rs (ids must match).
// Support levels come from docs/protocol/other-models.md ("Wire comparison").

import type { DeviceInfo } from "./model";
import type { KeyShape } from "./keys";
import type { Key } from "./i18n";
import C2_JSON from "./pads/C2.json";
import T3CE_JSON from "./pads/T3CE.json";
import G7_JSON from "./pads/G7ProCE.json";
import K2_JSON from "./pads/K2_8K.json";
import T3LITE_JSON from "./pads/T3Lite.json";
import T3PRO_JSON from "./pads/T3Pro.json";
import T4NPRO_JSON from "./pads/T4NPro.json";
import T4N2_JSON from "./pads/T4N2.json";
import N2S_JSON from "./pads/N2S.json";
import T3XS_JSON from "./pads/T3XS.json";

/** full: OpenSir configures it. codec: same link as the Cyclone 2, profile format not written yet. other: different protocol. */
export type Support = "full" | "codec" | "other";

/** A hotspot on the cover. `chip` = the key is not visible on the cover: drawn as a labelled pill. */
export interface PadSpot {
  x: number;
  y: number;
  shape: KeyShape;
  chip?: boolean;
}

/** Grip light strip of the C2 (ellipse). */
export interface LedSpot {
  /** LED slot in the light frame: 0 lower-left grip, 1 lower-right grip, 4 back. */
  slot: number;
  x: number;
  y: number;
  rx: number;
  ry: number;
  rot: number;
}

/** One controller cover: the official GameSir image and its hotspots, in a 1000-unit wide frame. */
export interface PadSpec {
  /** Served path of the cover SVG (static/pads/<id>.svg). */
  cover: string;
  /** Frame height: the cover's height in the same 1000-unit scale. */
  height: number;
  /** Hotspots by key id (every KEYS id). */
  spots: Record<string, PadSpot>;
  /** Spots for ModelInfo.extraKeys, by index. */
  extra: PadSpot[];
  /** Spots for the touch keys (T3CE Ext8, Ext9). Empty elsewhere. */
  touchKeys: PadSpot[];
  /** Grip light strips (C2 only). Slot 0 lower-left, slot 1 lower-right. */
  leds: LedSpot[];
  /** Drawn grey regardless of the cover (unknown models). */
  mono?: boolean;
}

/** Light controls of a V2-family controller: settings and zones it exposes, and the tail fields it keeps. */
export interface LightSpec {
  settings: string[];
  zones: string[];
  tail: string[];
}

export interface ModelInfo {
  id: string;
  name: string;
  support: Support;
  /** Protocol family: C2 (Cyclone 2), V2 (T3CE, G7 Pro 8K, K2 8K) or none. */
  family: "C2" | "V2" | "none";
  /** Light controls, V2 only. */
  light?: LightSpec;
  /** Labels of function keys beyond FL1/FR1, in profile order. */
  extraKeys: string[];
  pad: PadSpec;
  wired: number[];
  wireless: number[];
}

/** Pad from its measured JSON (src/lib/pads/<id>.json) and the cover served from static/pads/<id>.svg. */
function pad(id: string, json: object): PadSpec {
  return { cover: `/pads/${id}.svg`, ...json } as PadSpec;
}

const C2_PAD = pad("C2", C2_JSON);
const T3CE_PAD = pad("T3CE", T3CE_JSON);
const G7_PAD = pad("G7ProCE", G7_JSON);
const K2_PAD = pad("K2_8K", K2_JSON);
const T3LITE_PAD = pad("T3Lite", T3LITE_JSON);
const T3PRO_PAD = pad("T3Pro", T3PRO_JSON);
const T4NPRO_PAD = pad("T4NPro", T4NPRO_JSON);
const T4N2_PAD = pad("T4N2", T4N2_JSON);
const N2S_PAD = pad("N2S", N2S_JSON);
const T3XS_PAD = pad("T3XS", T3XS_JSON);

const V2_SETTINGS = [
  "singleColorBrightness", "breathingBrightness", "phantomBrightness", "rainbowBrightness", "radarBrightness",
  "breathingSpeed", "phantomSpeed", "rainbowSpeed", "radarSpeed", "phantomHue", "rainbowHue",
];
const T3CE_LIGHT: LightSpec = { settings: V2_SETTINGS, zones: ["panel", "home"], tail: ["standbyTime"] };
const G7_LIGHT: LightSpec = {
  settings: V2_SETTINGS,
  zones: ["lowerLeft", "upperLeft", "upperRight", "lowerRight", "home"],
  tail: ["standbyTime", "dockLedMode", "dockLedBrightness", "reportRate"],
};
const K2_LIGHT: LightSpec = {
  settings: ["homeRgbEffectIndex", "profileNumRgbEffectIndex", "homeRgbPhantomSpeed", "profileNumRgbPhantomSpeed", ...V2_SETTINGS],
  zones: ["lowerLeft", "lowerRight", "upperLeft", "upperRight", "home", "profileNum", "homeButton"],
  tail: ["standbyTime", "dockLedMode", "dockLedBrightness", "reportRate", "deviceId0", "deviceId1", "deviceId2", "deviceId3"],
};

export const MODELS: ModelInfo[] = [
  { id: "C2", name: "Cyclone 2", support: "full", family: "C2", extraKeys: [], pad: C2_PAD, wired: [0x101d, 0x1053], wireless: [0x102a, 0x100b] },
  { id: "T3CE", name: "Tarantula CE", support: "full", family: "V2", light: T3CE_LIGHT, extraKeys: ["Ext1", "Ext2", "Ext3", "Ext4", "Ext5", "Ext6", "Ext7"], pad: T3CE_PAD, wired: [0x103d, 0x090d], wireless: [] },
  {
    id: "G7ProCE", name: "G7 Pro 8K", support: "full", family: "V2", light: G7_LIGHT, extraKeys: ["FL2", "FR2"], pad: G7_PAD,
    wired: [0x10b7, 0x10cd, 0x10c5, 0x10fd, 0x10c7, 0x10c9, 0x10cb, 0x1032, 0x1034, 0x10fb],
    wireless: [0x10b9, 0x10ce, 0x10c6, 0x10fe, 0x10c8, 0x10ca, 0x10cc, 0x1033, 0x1035, 0x10fc],
  },
  { id: "K2_8K", name: "K2 8K", support: "full", family: "V2", light: K2_LIGHT, extraKeys: ["FL2", "FR2", "FL3", "FR3"], pad: K2_PAD, wired: [0x108e], wireless: [0x109f] },
  { id: "T3Lite", name: "Tegenaria Lite", support: "other", family: "none", extraKeys: [], pad: T3LITE_PAD, wired: [0x1093], wireless: [] },
  { id: "T3Pro", name: "Tarantula Pro", support: "other", family: "none", extraKeys: [], pad: T3PRO_PAD, wired: [0x1050], wireless: [] },
  { id: "T4NPro", name: "Nova Pro", support: "other", family: "none", extraKeys: [], pad: T4NPRO_PAD, wired: [0x100e], wireless: [] },
  { id: "T3Pro_T4NPro", name: "Tarantula Pro / Nova Pro", support: "other", family: "none", extraKeys: [], pad: T3PRO_PAD, wired: [], wireless: [0x103e] },
  { id: "T4N2", name: "Nova 2 Lite", support: "other", family: "none", extraKeys: [], pad: T4N2_PAD, wired: [0x100f], wireless: [0x1098] },
  { id: "N2S", name: "Nova Vision", support: "other", family: "none", extraKeys: [], pad: N2S_PAD, wired: [0x1141], wireless: [0x10bd] },
  { id: "T3XS", name: "T3XS", support: "other", family: "none", extraKeys: [], pad: T3XS_PAD, wired: [0x105a], wireless: [0x10be] },
];

// Unknown models: the C2 layout in neutral grey.
const UNKNOWN: ModelInfo = {
  id: "unknown", name: "GameSir controller", support: "other", family: "none", extraKeys: [],
  pad: { ...C2_PAD, mono: true },
  wired: [], wireless: [],
};

export function modelOf(d: DeviceInfo): ModelInfo {
  return MODELS.find((m) => m.id === d.model) ?? UNKNOWN;
}

/**
 * Devices worth a card, configurable ones first. A non-config entry is dropped only when it
 * provably belongs to a configurable controller (same USB serial). The Cyclone 2's game-mode
 * PID 0x0575 reports no serial, so it keeps its own entry; see `parkedDevices`.
 */
export function visibleDevices(list: DeviceInfo[]): DeviceInfo[] {
  const configSerials = new Set(list.filter((d) => d.configCapable && d.serial).map((d) => d.serial));
  const rank = (d: DeviceInfo) => (d.configCapable ? 0 : d.model === "C2" ? 1 : modelOf(d).support === "codec" ? 2 : 3);
  return list
    .filter((d) => d.configCapable || !d.serial || !configSerials.has(d.serial))
    .sort((a, b) => rank(a) - rank(b));
}

/**
 * Serial-less game-mode Cyclone 2 entries next to a configurable Cyclone 2. Usually that same
 * pad's other interface, but not provably, so the list tucks them away instead of hiding them.
 */
export function parkedDevices(list: DeviceInfo[]): DeviceInfo[] {
  if (!list.some((d) => d.configCapable && d.model === "C2")) return [];
  return list.filter((d) => !d.configCapable && !d.serial && d.model === "C2");
}

/** Status line of a device card: what the user can do with it. */
export function deviceStatus(d: DeviceInfo): { key: Key; tone: "ok" | "warn" | "off" } {
  if (d.configCapable) return { key: "statusReady", tone: "ok" };
  if (d.model === "C2") return { key: "notConfigCapable", tone: "warn" };
  return { key: modelOf(d).support === "codec" ? "statusCodec" : "statusOther", tone: "off" };
}

export const pidHex = (pid: number) => `0x${pid.toString(16).toUpperCase().padStart(4, "0")}`;
export const isWireless = (d: DeviceInfo) => d.mode === "wireless" || d.mode === "wirelessAdc";

const RESTRICTED_PIDS: Record<string, number[]> = { T3CE: [0x090d], G7ProCE: [0x10cd, 0x10ce] };

/** Restricted PIDs: keyboard/mouse targets, turbo and the Motion page are locked. */
export function isRestricted(d: DeviceInfo): boolean {
  return RESTRICTED_PIDS[d.model]?.includes(d.pid) ?? false;
}
