import { api, exportProfile, importProfile, type ResetScope } from "./api";
import type { DeviceInfo, DeviceSnapshot, EffectLight, GamepadState, LightProfile, Profile } from "./model";
import { isRestricted, modelOf, MODELS, type ModelInfo } from "./models";
import { withViewTransition } from "./motion";
import { lightSel } from "./lightSel.svelte";

/** V2 dead zones are 0..1000 on the device; the editor works in percent like the Cyclone 2. */
function deadZones(p: Profile) {
  return [p.l2.dead, p.r2.dead, p.leftStick.dead, p.rightStick.dead, p.aim.dead, p.tilt.dead];
}
function toPercent(p: Profile): Profile {
  for (const d of deadZones(p)) {
    d.front /= 10;
    d.back /= 10;
    d.antiFront /= 10;
    d.antiBack /= 10;
  }
  return p;
}
function toDevice(p: Profile): Profile {
  for (const d of deadZones(p)) {
    d.front = Math.round(d.front * 10);
    d.back = Math.round(d.back * 10);
    d.antiFront = Math.round(d.antiFront * 10);
    d.antiBack = Math.round(d.antiBack * 10);
  }
  return p;
}

export type View = "overview" | "settings" | "mapping" | "sticks" | "triggers" | "motion" | "motors" | "lights" | "buttons";
export type SaveStatus = "idle" | "saving" | "saved" | "error";

/** Trims a profile name to 32 UTF-8 bytes without splitting a character. */
export function clampName(raw: string): string {
  const enc = new TextEncoder();
  let out = "";
  let bytes = 0;
  for (const ch of raw) {
    const n = enc.encode(ch).length;
    if (bytes + n > 32) break;
    out += ch;
    bytes += n;
  }
  return out;
}

interface Job {
  run: () => Promise<void>;
  timer: ReturnType<typeof setTimeout>;
}

class App {
  devices = $state<DeviceInfo[]>([]);
  openId = $state<string | null>(null);
  snapshot = $state<DeviceSnapshot | null>(null);
  viewIndex = $state(1);
  view = $state<View>("overview");
  status = $state<SaveStatus>("idle");
  error = $state("");
  gamepad = $state<GamepadState | null>(null);
  theme = $state<"dark" | "light">("dark");
  locale = $state<"en" | "pt-BR">("en");

  private jobs = new Map<string, Job>();
  private chain: Promise<void> = Promise.resolve();

  get profile(): Profile | null {
    return this.snapshot?.profiles[this.viewIndex - 1] ?? null;
  }

  /** Model of the open device; the Cyclone 2 drawing when nothing is open. */
  get model(): ModelInfo {
    const d = this.devices.find((x) => x.id === this.openId);
    return d ? modelOf(d) : MODELS[0];
  }

  get light(): LightProfile | null {
    return this.snapshot?.light ?? null;
  }

  get effectLight(): EffectLight | null {
    return this.snapshot?.effectLight ?? null;
  }

  /** False for V2-family devices: they open read-only, so the editor's flash writes stay off here too. */
  get writable(): boolean {
    return this.snapshot?.writable ?? false;
  }

  /** Restricted PIDs: no keyboard/mouse targets, no turbo, no motion page. */
  get restricted(): boolean {
    const d = this.devices.find((x) => x.id === this.openId);
    return d ? isRestricted(d) : false;
  }

  /** Device profile -> editor profile (V2 dead zones to percent). */
  private fromDevice(p: Profile, model = this.model): Profile {
    return model.family === "V2" ? toPercent(p) : p;
  }

  /** Editor profile -> device profile (a copy; V2 dead zones back to 0..1000). */
  private forDevice(p: Profile, model = this.model): Profile {
    const copy = $state.snapshot(p) as Profile;
    return model.family === "V2" ? toDevice(copy) : copy;
  }

  async init() {
    await this.refreshDevices();
    await api.listen("device-list-changed", (list) => {
      this.devices = list;
      if (this.openId && !list.some((d) => d.id === this.openId)) void this.close();
    });
    await api.listen("device-state", (s) => {
      if (s.id !== this.openId || !this.snapshot) return;
      this.snapshot.currentProfile = s.currentProfile;
      this.snapshot.battery = s.battery;
      this.snapshot.charging = s.charging;
    });
    await api.listen("gamepad-state", (g) => (this.gamepad = g));
    await api.listen("device-error", (e) => {
      if (e.id !== this.openId) return;
      // The backend already dropped the session: leave the editor instead of failing every edit.
      for (const key of [...this.jobs.keys()]) this.cancel(key);
      this.openId = null;
      this.snapshot = null;
      this.gamepad = null;
      this.fail(e.message);
    });
  }

  async refreshDevices() {
    await this.call(async () => {
      this.devices = await api.listDevices();
    });
  }

  async open(id: string) {
    await this.call(async () => {
      const snap = await api.openDevice(id);
      // openId is set only in the commit below, so convert with the opening device's own model.
      const dev = this.devices.find((x) => x.id === id);
      const model = dev ? modelOf(dev) : MODELS[0];
      snap.profiles = snap.profiles.map((p) => this.fromDevice(p, model));
      if (snap.shift) snap.shift = this.fromDevice(snap.shift, model);
      // One commit, so the list → device swap happens in a single view transition.
      await withViewTransition(() => {
        this.openId = id;
        this.snapshot = snap;
        this.viewIndex = snap.currentProfile;
        this.view = "overview";
        this.error = "";
        this.status = "idle";
        // The C2 plays the animation in its ConfigIndex: start the slot tabs there.
        lightSel.slot = snap.light?.configIndex ?? 0;
      });
    });
  }

  async close() {
    await this.flush();
    const id = this.openId;
    await withViewTransition(() => {
      this.openId = null;
      this.snapshot = null;
      this.gamepad = null;
    });
    if (id) await this.enqueue(() => api.closeDevice(id));
  }

  /** Vendor behaviour: the open device expects a heartbeat each second while the window has focus. */
  keepAlive(): () => void {
    const timer = setInterval(() => {
      const id = this.openId;
      if (id && document.hasFocus()) void this.call(() => api.heartbeat(id));
    }, 1000);
    return () => clearInterval(timer);
  }

  /** Saves the profile currently shown in the editor (debounced). */
  saveProfile() {
    this.saveIndex(this.viewIndex);
  }

  saveLight() {
    const id = this.openId;
    const light = this.light;
    if (!id || !light || !this.writable) return;
    this.schedule("light", () => api.saveLight(id, $state.snapshot(light)));
  }

  saveEffectLight() {
    const id = this.openId;
    const light = this.effectLight;
    if (!id || !light || !this.writable) return;
    this.schedule("effectLight", () => api.saveEffectLight(id, $state.snapshot(light)));
  }

  setName(raw: string) {
    if (!this.profile) return;
    this.profile.name = clampName(raw);
    this.saveProfile();
  }

  async activate() {
    await this.flush();
    const id = this.openId;
    const snap = this.snapshot;
    const index = this.viewIndex;
    if (!id || !snap || !this.writable) return;
    await this.enqueue(async () => {
      await api.switchProfile(id, index);
      snap.currentProfile = index;
      this.status = "saved";
    });
  }

  async reset(scope: ResetScope) {
    const id = this.openId;
    const snap = this.snapshot;
    const index = this.viewIndex;
    if (!id || !snap || !this.writable) return;
    this.cancel(this.profileKey(index));
    await this.enqueue(async () => {
      snap.profiles[index - 1] = this.fromDevice(await api.resetProfile(id, index, scope));
      this.status = "saved";
    });
  }

  async exportCurrent() {
    const p = this.profile;
    if (!p) return;
    await this.call(() => exportProfile(this.forDevice(p), this.model.id));
  }

  async importInto() {
    const snap = this.snapshot;
    const index = this.viewIndex;
    if (!snap || !this.writable) return;
    await this.call(async () => {
      const p = await importProfile(this.model.id);
      if (!p) return;
      snap.profiles[index - 1] = this.fromDevice(p);
      this.saveIndex(index);
    });
  }

  /** Pending saves (e.g. a new ConfigIndex or preset frames) go out first, so the device plays what the editor shows. */
  async setPlayback(playing: boolean, frame: number) {
    await this.flush();
    const id = this.openId;
    if (id) await this.enqueue(() => api.setLightPlayback(id, playing, frame));
  }

  vibrate(left: number, right: number, durationMs: number) {
    const id = this.openId;
    if (id) void this.call(() => api.vibrate(id, left, right, durationMs));
  }

  setTestMode(on: boolean) {
    const id = this.openId;
    if (id) void this.call(() => api.setTestMode(id, on));
  }

  private saveIndex(index: number) {
    const id = this.openId;
    const profile = this.snapshot?.profiles[index - 1];
    if (!id || !profile || !this.writable) return;
    // Model fixed now: if the device leaves the list before the save runs, `this.model` falls back to the Cyclone 2.
    const model = this.model;
    this.schedule(this.profileKey(index), () => api.saveProfile(id, index, this.forDevice(profile, model)));
  }

  private profileKey(index: number): string {
    return `profile:${index}`;
  }

  private schedule(key: string, run: () => Promise<void>) {
    this.cancel(key);
    const timer = setTimeout(() => void this.runJob(key), 300);
    this.jobs.set(key, { run, timer });
  }

  private cancel(key: string) {
    const job = this.jobs.get(key);
    if (!job) return;
    clearTimeout(job.timer);
    this.jobs.delete(key);
  }

  /** Waits for every queued save, including the one already running. */
  private async flush() {
    for (const key of [...this.jobs.keys()]) this.runJob(key);
    await this.chain;
  }

  private runJob(key: string) {
    const job = this.jobs.get(key);
    if (!job) return;
    clearTimeout(job.timer);
    this.jobs.delete(key);
    void this.enqueue(async () => {
      this.status = "saving";
      await job.run();
      this.status = "saved";
    });
  }

  /** Device writes run one at a time in queue order, so a later write never overtakes an earlier one. */
  private enqueue(job: () => Promise<unknown>): Promise<void> {
    this.chain = this.chain.then(() => this.call(job));
    return this.chain;
  }

  private async call(job: () => Promise<unknown>) {
    try {
      await job();
    } catch (e) {
      this.fail(e);
    }
  }

  private fail(e: unknown) {
    this.status = "error";
    this.error = e instanceof Error ? e.message : String(e);
  }
}

export const app = new App();
