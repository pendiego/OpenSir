// Hand-written mirror of crates/opensir-core/src/model.rs (serde camelCase).
// Keep in sync by hand; `Profile.raw` is `#[serde(skip)]` and therefore absent.

export interface Button {
  turbo: number;
  mapped: boolean;
  map: [number, number, number];
  toggleOnPress: boolean;
}

export interface MacroStep {
  key: number;
  holdMs: number;
  delayMs: number;
}

export interface FunctionKey {
  turbo: number;
  mapped: boolean;
  map: [number, number, number];
  toggleOnPress: boolean;
  macroEnabled: boolean;
  macroPressToTrigger: boolean;
  macroCycle: boolean;
  cycleMs: number;
  steps: MacroStep[];
}

export interface DeadZone {
  enabled: boolean;
  front: number;
  back: number;
  antiFront: number;
  antiBack: number;
}

export interface Linear {
  enabled: boolean;
  curveType: number;
  curvature: number;
  points: [[number, number], [number, number], [number, number], [number, number], [number, number]];
}

export interface Trigger {
  turbo: number;
  dead: DeadZone;
  mapped: boolean;
  map: [number, number, number];
  toggleOnPress: boolean;
  quickTriggerStatus: number;
  quickTriggerStart: number;
  quickTriggerEnd: number;
  linear: Linear;
}

export interface MapModule {
  enabled: boolean;
  xFlip: boolean;
  yFlip: boolean;
  /** V2 motion only. */
  zFlip: boolean;
  axisRatio: number;
  mouseDpi: number;
  mapIndex: number;
  mapCross: number;
  mapUp: number;
  mapDown: number;
  mapLeft: number;
  mapRight: number;
  mapDead: number;
}

export interface Stick {
  enabled: boolean;
  square: boolean;
  dead: DeadZone;
  linear: Linear;
  map: MapModule;
}

export interface Motion {
  sensorProfileStatus: number;
  sensorQuickKeyValue: number;
  /** V2 only (0 on the Cyclone 2). */
  quickKeyValue2: number;
  quickKeyValue3: number;
  activeAxis: number;
  dead: DeadZone;
  linear: Linear;
  map: MapModule;
  /** V2 only. */
  stickSensitivity: number;
}

export interface Fun {
  motors: [number, number, number, number];
  audioEnabled: number;
  audioVolume: number;
  audioMixer: number;
  micMute: number;
  micSensitivity: number;
  shiftEnabled: number;
  shiftValue: number;
  dpadDiagonalLock: number;
  xinputAbxyChange: number;
  switchAbxyChange: number;
  reportRateGears: number;
  /** V2 only. */
  sensorQuickMode: number;
  stickResolution: number;
  /** G7ProCE and K2_8K only. */
  lsAntiJitter: number;
  rsAntiJitter: number;
}

/** T3CE touch keys Ext8/Ext9. */
export interface TouchKey {
  turbo: number;
  mapped: boolean;
  map: [number, number, number];
  toggleOnPress: boolean;
  triggerMode: number;
  sensitivity: number;
}

export interface Profile {
  name: string;
  fun: Fun;
  buttons: [Button, Button, Button, Button, Button, Button, Button, Button,
    Button, Button, Button, Button, Button, Button, Button, Button];
  fl1: FunctionKey;
  fr1: FunctionKey;
  /** Function keys after FL1/FR1, named by `ModelInfo.extraKeys`. */
  extra: FunctionKey[];
  /** T3CE only. */
  touch: TouchKey[];
  l2: Trigger;
  r2: Trigger;
  leftStick: Stick;
  rightStick: Stick;
  aim: Motion;
  tilt: Motion;
}

/** V2 effect light (T3CE, G7ProCE, K2_8K). Names of `settings`, `zones` and `tail` per model: `ModelInfo.light`. */
export interface Hsv {
  /** 0..359 */
  h: number;
  /** 0..100 */
  s: number;
  v: number;
}

export interface EffectAnimation {
  mode: number;
  settings: number[];
  zones: Hsv[];
}

export interface EffectLight {
  animations: EffectAnimation[];
  raiseWakeUp: boolean;
  tail: number[];
}

type RGB = [number, number, number];

export interface LightAnimation {
  keyFrameCount: number;
  effectCount: number;
  speed: number;
  brightness: number;
  frames: [RGB, RGB, RGB, RGB, RGB][];
}

export interface LightProfile {
  configIndex: number;
  animations: LightAnimation[];
  audioReactive: boolean;
  userEffectIndex: number;
  profileLed: RGB;
  raiseWakeUp: boolean;
  standbyTime: number;
}

export interface GamepadState {
  id: string;
  buttons: string[];
  lx: number;
  ly: number;
  rx: number;
  ry: number;
  lt: number;
  rt: number;
}

export type DeviceMode = "wired" | "wireless" | "wiredAdc" | "wirelessAdc" | "unsupported";

export interface DeviceInfo {
  id: string;
  model: string;
  name: string;
  vid: number;
  pid: number;
  mode: DeviceMode;
  configCapable: boolean;
  /** USB serial number, when reported. Equal serials = one physical controller. */
  serial: string | null;
}

export interface DeviceSnapshot {
  id: string;
  /** Model id from models.ts, e.g. "C2", "G7ProCE". */
  model: string;
  firmware: string;
  dongleFirmware: string | null;
  currentProfile: number;
  profiles: Profile[];
  shift: Profile | null;
  /** Cyclone 2 light. */
  light: LightProfile | null;
  /** V2 family light. */
  effectLight: EffectLight | null;
  battery: number | null;
  charging: boolean | null;
  /** False for V2-family models: OpenSir reads them but never writes until they are tested on hardware. */
  writable: boolean;
}
