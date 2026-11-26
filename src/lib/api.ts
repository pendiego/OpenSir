import { invoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import type { DeviceInfo, DeviceSnapshot, EffectLight, GamepadState, LightProfile, Profile } from "./model";
import { mockBackend } from "./mock";

export type ResetScope = "all" | "mapping" | "sticks" | "motion" | "triggers" | "motors";

export interface DeviceState {
  id: string;
  currentProfile: number;
  battery: number | null;
  charging: boolean | null;
}

export interface DeviceError {
  id: string;
  message: string;
}

export interface EventMap {
  "device-list-changed": DeviceInfo[];
  "device-state": DeviceState;
  "gamepad-state": GamepadState;
  "device-error": DeviceError;
}

export type EventName = keyof EventMap;

export interface Backend {
  listDevices(): Promise<DeviceInfo[]>;
  openDevice(id: string): Promise<DeviceSnapshot>;
  closeDevice(id: string): Promise<void>;
  saveProfile(id: string, index: number, profile: Profile): Promise<void>;
  switchProfile(id: string, index: number): Promise<void>;
  resetProfile(id: string, index: number, scope: ResetScope): Promise<Profile>;
  saveLight(id: string, light: LightProfile): Promise<void>;
  saveEffectLight(id: string, light: EffectLight): Promise<void>;
  setLightPlayback(id: string, playing: boolean, frame: number): Promise<void>;
  vibrate(id: string, left: number, right: number, durationMs: number): Promise<void>;
  heartbeat(id: string): Promise<void>;
  setTestMode(id: string, on: boolean): Promise<void>;
  listen<K extends EventName>(event: K, cb: (payload: EventMap[K]) => void): Promise<() => void>;
}

export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const tauriBackend: Backend = {
  listDevices: () => invoke<DeviceInfo[]>("list_devices"),
  openDevice: (id) => invoke<DeviceSnapshot>("open_device", { id }),
  closeDevice: (id) => invoke<void>("close_device", { id }),
  saveProfile: (id, index, profile) => invoke<void>("save_profile", { id, index, profile }),
  switchProfile: (id, index) => invoke<void>("switch_profile", { id, index }),
  resetProfile: (id, index, scope) => invoke<Profile>("reset_profile", { id, index, scope }),
  saveLight: (id, light) => invoke<void>("save_light", { id, light }),
  saveEffectLight: (id, light) => invoke<void>("save_effect_light", { id, light }),
  setLightPlayback: (id, playing, frame) => invoke<void>("set_light_playback", { id, playing, frame }),
  vibrate: (id, left, right, durationMs) => invoke<void>("vibrate", { id, left, right, durationMs }),
  heartbeat: (id) => invoke<void>("heartbeat", { id }),
  setTestMode: (id, on) => invoke<void>("set_test_mode", { id, on }),
  listen: <K extends EventName>(event: K, cb: (payload: EventMap[K]) => void) =>
    tauriListen<EventMap[K]>(event, (e) => cb(e.payload)),
};

export const api: Backend = isTauri ? tauriBackend : mockBackend;

const PROFILE_FORMAT = "opensir-profile";

interface ProfileDoc {
  format: string;
  version: number;
  model: string;
  profile: Profile;
}

function parseProfileDoc(text: string, model: string): Profile {
  const doc = JSON.parse(text) as ProfileDoc;
  if (doc.format !== PROFILE_FORMAT || doc.version !== 1 || doc.model !== model) {
    throw new Error(`not an OpenSir profile file for ${model}`);
  }
  return doc.profile;
}

/** Writes the profile, tagged with its model id, to a file chosen by the user (Tauri) or downloads it (browser). */
export async function exportProfile(profile: Profile, model: string): Promise<void> {
  if (isTauri) {
    const path = await saveDialog({
      defaultPath: `${profile.name || "profile"}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (path) await invoke<void>("export_profile", { profile, path, model });
    return;
  }
  const doc: ProfileDoc = { format: PROFILE_FORMAT, version: 1, model, profile };
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([JSON.stringify(doc, null, 2)], { type: "application/json" }));
  a.download = `${profile.name || "profile"}.json`;
  a.click();
  URL.revokeObjectURL(a.href);
}

/** Reads a profile the user picks. Resolves null when the user cancels. */
export async function importProfile(model: string): Promise<Profile | null> {
  if (isTauri) {
    const path = await openDialog({ multiple: false, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (typeof path !== "string") return null;
    return invoke<Profile>("import_profile", { path, model });
  }
  return new Promise((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return resolve(null);
      try {
        resolve(parseProfileDoc(await file.text(), model));
      } catch (e) {
        reject(e);
      }
    };
    input.click();
  });
}
