// Browser-preview backend. Simulates GameSir controllers in memory (plug/unplug included); never touches USB.
import { MODELS } from "./models";
import type { Backend, EventMap, EventName, ResetScope } from "./api";
import type {
  Button,
  DeadZone,
  DeviceInfo,
  DeviceSnapshot,
  Frame,
  FunctionKey,
  GamepadState,
  LightProfile,
  EffectLight,
  Linear,
  MapModule,
  Motion,
  Profile,
  Stick,
  Trigger,
} from "./model";

const deadZone = (front: number, back: number, antiFront: number, antiBack: number): DeadZone => ({
  enabled: true,
  front,
  back,
  antiFront,
  antiBack,
});

const linear = (curveType = 0, curvature = 100): Linear => ({
  enabled: false,
  curveType,
  curvature,
  // Wire layout (see $lib/curve): fixed ends, three points relative to the deadzone.
  points: [[0, 0], [64, 64], [128, 128], [191, 191], [255, 255]],
});

const map = (mapIndex: number): MapModule => ({
  enabled: false,
  xFlip: false,
  zFlip: false,
  yFlip: false,
  axisRatio: 50,
  mouseDpi: 50,
  mapIndex,
  mapCross: 0,
  mapUp: 0,
  mapDown: 0,
  mapLeft: 0,
  mapRight: 0,
  mapDead: 0,
});

const button = (): Button => ({ turbo: 0, mapped: false, map: [0, 0, 0], toggleOnPress: false });
const fnKey = (): FunctionKey => ({
  ...button(),
  macroEnabled: false,
  macroPressToTrigger: false,
  macroCycle: false,
  cycleMs: 0,
  steps: [],
});
const trigger = (): Trigger => ({
  ...button(),
  dead: deadZone(5, 95, 0, 100),
  quickTriggerStatus: 0,
  quickTriggerStart: 1,
  quickTriggerEnd: 100,
  linear: linear(),
});
const stick = (mapIndex: number): Stick => ({
  enabled: false,
  square: false,
  dead: deadZone(10, 100, 0, 100),
  linear: linear(),
  map: map(mapIndex),
});
const motion = (mapIndex: number): Motion => ({
  sensorProfileStatus: 0,
  sensorQuickKeyValue: 255, // NC, as reset writes
  quickKeyValue2: 0,
  quickKeyValue3: 0,
  stickSensitivity: 0,
  activeAxis: 2, // yaw (bit 1)
  dead: deadZone(10, 100, 0, 100),
  linear: linear(),
  map: map(mapIndex),
});

export function defaultProfile(name: string, extraCount = 0): Profile {
  return {
    name,
    fun: {
      motors: [3, 3, 3, 3],
      audioEnabled: 0,
      audioVolume: 0,
      audioMixer: 0,
      micMute: 0,
      micSensitivity: 0,
      shiftEnabled: 0,
      shiftValue: 0,
      dpadDiagonalLock: 0,
      sensorQuickMode: 0,
      stickResolution: 0,
      lsAntiJitter: 0,
      rsAntiJitter: 0,
      xinputAbxyChange: 0,
      switchAbxyChange: 0,
      reportRateGears: 2,
    },
    buttons: Array.from({ length: 16 }, () => button()) as Profile["buttons"],
    fl1: fnKey(),
    fr1: fnKey(),
    extra: Array.from({ length: extraCount }, () => fnKey()),
    touch: [],
    l2: trigger(),
    r2: trigger(),
    leftStick: stick(1),
    rightStick: stick(2),
    aim: motion(1),
    tilt: motion(2),
  };
}

const frame = (): Frame => [
  [255, 0, 0],
  [0, 0, 255],
  [0, 0, 0],
  [0, 255, 0],
  [255, 255, 255],
];

// Browser preview only: V2 pads carry a fixed-size effect light sized by their model; C2 has none.
function defaultEffectLight(model: string): EffectLight | null {
  const spec = MODELS.find((m) => m.id === model)?.light;
  if (!spec) return null;
  return {
    animations: Array.from({ length: 5 }, (_, slot) => ({
      mode: slot,
      settings: spec.settings.map((_name, i) => 40 + ((i * 17 + slot * 9) % 60)),
      zones: spec.zones.map((_zone, z) => ({ h: (z * 97 + slot * 60) % 360, s: 100, v: 80 })),
    })),
    raiseWakeUp: false,
    tail: spec.tail.map((_name, i) => (i === 0 ? 30 : 0)),
  } as EffectLight;
}

export function defaultLight(): LightProfile {
  return {
    configIndex: 0,
    animations: Array.from({ length: 5 }, () => ({
      keyFrameCount: 2,
      effectCount: 1,
      speed: 10,
      brightness: 100,
      // Like the device: always 8 frames; keyFrameCount says how many play.
      frames: Array.from({ length: 8 }, () => frame()),
    })),
    audioReactive: false,
    userEffectIndex: 0,
    profileLed: [0, 0, 0],
    raiseWakeUp: false,
    standbyTime: 0,
  };
}

// Reset values from the device reset table (spec section 1.3).
function unmap(k: Button): void {
  Object.assign(k, { turbo: 0, mapped: false, map: [0, 0, 0], toggleOnPress: false });
}

function zeroMap(m: MapModule): void {
  Object.assign(m, {
    xFlip: false,
    yFlip: false,
    mapCross: 0,
    axisRatio: 50,
    mouseDpi: 50,
    mapUp: 0,
    mapDown: 0,
    mapLeft: 0,
    mapRight: 0,
    mapDead: 0,
  });
}

// `k` scales dead zones to device units: 1 for the Cyclone 2 (0..100), 10 for the V2 family (0..1000).
const RESET: Record<Exclude<ResetScope, "all">, (p: Profile, k: number) => void> = {
  mapping: (p) => {
    p.buttons.forEach(unmap);
    unmap(p.l2);
    unmap(p.r2);
    for (const k of [p.fl1, p.fr1, ...p.extra]) {
      unmap(k);
      Object.assign(k, { macroEnabled: false, macroPressToTrigger: false, macroCycle: false, cycleMs: 0, steps: [] });
    }
    p.touch.forEach(unmap);
  },
  sticks: (p, k) => {
    for (const s of [p.leftStick, p.rightStick]) {
      Object.assign(s, { enabled: false, dead: deadZone(10 * k, 100 * k, 0, 100 * k), linear: linear() });
      zeroMap(s.map);
    }
  },
  motion: (p, k) => {
    for (const m of [p.aim, p.tilt]) {
      Object.assign(m, {
        sensorProfileStatus: 0,
        sensorQuickKeyValue: 255,
        activeAxis: 2,
        dead: deadZone(10 * k, 100 * k, 0, 100 * k),
        linear: linear(),
      });
      zeroMap(m.map);
    }
  },
  triggers: (p, k) => {
    // shortcut: hair-trigger mode is not reset (device stores it separately), upgrade when the mode field is mapped.
    for (const t of [p.l2, p.r2]) {
      Object.assign(t, {
        dead: deadZone(5 * k, 95 * k, 0, 100 * k),
        linear: linear(),
        quickTriggerStatus: 0,
        quickTriggerStart: 1,
        quickTriggerEnd: 100,
      });
    }
  },
  motors: (p) => {
    p.fun.motors = [3, 3, 3, 3];
  },
};

function applyReset(p: Profile, scope: ResetScope, k: number): void {
  const groups = scope === "all" ? (Object.keys(RESET) as (keyof typeof RESET)[]) : [scope];
  for (const g of groups) RESET[g](p, k);
}

const isV2 = (model: string) => MODELS.find((m) => m.id === model)?.family === "V2";

function mockSnapshot(id: string, battery: number | null, model: string): DeviceSnapshot {
  const info = MODELS.find((m) => m.id === model);
  const profiles = [1, 2, 3, 4].map((n) => defaultProfile(`Profile ${n}`, info?.extraKeys.length ?? 0));
  if (isV2(model)) {
    // Like the device: V2 dead zones in 0..1000, T3CE touch keys Ext8/Ext9.
    for (const p of profiles) {
      for (const d of [p.l2.dead, p.r2.dead, p.leftStick.dead, p.rightStick.dead, p.aim.dead, p.tilt.dead]) {
        Object.assign(d, { front: d.front * 10, back: d.back * 10, antiFront: d.antiFront * 10, antiBack: d.antiBack * 10 });
      }
      if (info?.pad.touchKeys) {
        p.touch = [0, 1].map(() => ({ turbo: 0, mapped: false, map: [0, 0, 0], toggleOnPress: false, triggerMode: 1, sensitivity: 600 }));
      }
    }
  }
  return {
    id,
    model,
    firmware: "3.4.0",
    dongleFirmware: null,
    currentProfile: 1,
    profiles,
    shift: null,
    light: isV2(model) ? null : defaultLight(),
    effectLight: defaultEffectLight(model),
    battery,
    charging: false,
    writable: !isV2(model),
  };
}

// Same names and PIDs the real backend reports (crates/opensir-core/src/models.rs).
const devices: DeviceInfo[] = [
  { id: "mock-c2", model: "C2", name: "GameSir Cyclone 2", vid: 0x3537, pid: 0x101d, mode: "wired", configCapable: true, serial: "SIM-C2-0001" },
  { id: "mock-g7", model: "G7ProCE", name: "GameSir G7 Pro 8K", vid: 0x3537, pid: 0x10b9, mode: "wireless", configCapable: true, serial: "SIM-G7-0001" },
  { id: "mock-n2l", model: "T4N2", name: "GameSir Nova 2 Lite", vid: 0x3537, pid: 0x100f, mode: "wired", configCapable: true, serial: "SIM-N2L-0001" },
];

const snapshots = new Map<string, DeviceSnapshot>(
  devices
    .filter((d) => d.configCapable)
    .map((d): [string, DeviceSnapshot] => [d.id, mockSnapshot(d.id, d.id === "mock-c2" ? 80 : d.mode === "wireless" ? 64 : null, d.model)]),
);

/** What the preview can plug in: every model by cable or wireless, plus a Cyclone 2 left in game (non-config) mode. */
export const SIMULATED = [
  ...MODELS.flatMap((m) => [
    ...(m.wired.length ? [{ key: `${m.id}:wired`, label: `${m.name} · USB` }] : []),
    ...(m.wireless.length ? [{ key: `${m.id}:wireless`, label: `${m.name} · 2.4 GHz` }] : []),
  ]),
  { key: "C2:game", label: "Cyclone 2 · game mode (0x0575)" },
];

let nextId = 1;

/** Browser preview only: plug a simulated controller in, the way the hotplug watcher reports it. */
export function simulatePlug(key: string): void {
  const [modelId, how] = key.split(":");
  const m = MODELS.find((x) => x.id === modelId);
  if (!m) return;
  const id = `mock-${modelId}-${nextId++}`;
  const game = how === "game";
  const wireless = how === "wireless";
  // Mock pads open even where the real protocol does not configure them yet.
  const config = !game;
  devices.push({
    id,
    model: m.id,
    name: `GameSir ${m.name}`,
    vid: 0x3537,
    pid: game ? 0x0575 : (wireless ? m.wireless : m.wired)[0],
    mode: game ? "unsupported" : wireless ? "wireless" : "wired",
    configCapable: config,
    // Every simulated pad is its own controller; game mode reports no serial, like the real 0x0575.
    serial: game ? null : `SIM-${id}`,
  });
  if (config) snapshots.set(id, mockSnapshot(id, wireless ? 64 : null, m.id));
  emit("device-list-changed", structuredClone(devices));
}

/** Browser preview only: unplug a simulated controller. */
export function simulateUnplug(id: string): void {
  const i = devices.findIndex((d) => d.id === id);
  if (i < 0) return;
  devices.splice(i, 1);
  emit("device-list-changed", structuredClone(devices));
}

function snap(id: string): DeviceSnapshot {
  const s = snapshots.get(id);
  if (!s) throw new Error("device is not in config mode");
  return s;
}

type Listener = (payload: unknown) => void;
const subs = new Map<EventName, Set<Listener>>();

function emit<K extends EventName>(event: K, payload: EventMap[K]): void {
  for (const cb of subs.get(event) ?? []) cb(payload);
}

let testTimer: ReturnType<typeof setInterval> | undefined;
let tick = 0;
const PAD_NAMES = ["CRO", "CIR", "SQU", "TRI", "L1", "R1", "Up", "Under", "Left", "Right", "Sel", "Sta"];

function pad(): GamepadState {
  tick += 0.05;
  const s = Math.sin(tick);
  const c = Math.cos(tick);
  return {
    id: "mock-c2",
    buttons: [PAD_NAMES[Math.floor(tick) % PAD_NAMES.length]],
    lx: s,
    ly: c,
    rx: c,
    ry: s,
    lt: Math.round(((s + 1) / 2) * 255),
    rt: Math.round(((c + 1) / 2) * 255),
  };
}

export const mockBackend: Backend = {
  async listDevices() {
    return structuredClone(devices);
  },
  async openDevice(id) {
    if (!devices.find((d) => d.id === id)?.configCapable) throw new Error("device is not in config mode");
    return structuredClone(snap(id));
  },
  async closeDevice() {},
  async saveProfile(id, index, profile) {
    snap(id).profiles[index - 1] = structuredClone(profile);
  },
  async switchProfile(id, index) {
    const s = snap(id);
    s.currentProfile = index;
    emit("device-state", { id, currentProfile: index, battery: s.battery, charging: s.charging });
  },
  async resetProfile(id, index, scope) {
    const s = snap(id);
    const p = s.profiles[index - 1];
    applyReset(p, scope, isV2(s.model) ? 10 : 1);
    return structuredClone(p);
  },
  async saveLight(id, light) {
    snap(id).light = structuredClone(light);
  },
  async saveEffectLight(id, light) {
    snap(id).effectLight = structuredClone(light);
  },
  // Browser preview has no device: playback and vibration are UI-only here.
  async setLightPlayback() {},
  async vibrate() {},
  async heartbeat() {},
  async setTestMode(_id: string, on: boolean) {
    clearInterval(testTimer);
    testTimer = undefined;
    if (on) testTimer = setInterval(() => emit("gamepad-state", pad()), 50);
  },
  async listen<K extends EventName>(event: K, cb: (payload: EventMap[K]) => void) {
    const set = subs.get(event) ?? new Set<Listener>();
    subs.set(event, set);
    const fn = cb as Listener;
    set.add(fn);
    return () => {
      set.delete(fn);
    };
  },
};
