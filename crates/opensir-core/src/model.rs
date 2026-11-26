//! Typed profile model. Field names follow `docs/protocol/cyclone2-profile.md`.
//! Values are raw protocol values unless a doc comment says otherwise. JSON is camelCase.

use serde::{Deserialize, Serialize};

/// One mapped button slot (16 of them). Packet layout: `profile.md` section 3.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Button {
    /// Turbo rate; 0 when turbo is off.
    pub turbo: u8,
    pub mapped: bool,
    /// Key codes (see `keycodes`), 0 = unused.
    pub map: [u8; 3],
    pub toggle_on_press: bool,
}

/// FL1/FR1 function key with an optional macro (159-byte packet, section 4).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionKey {
    pub turbo: u8,
    pub mapped: bool,
    pub map: [u8; 3],
    pub toggle_on_press: bool,
    pub macro_enabled: bool,
    pub macro_press_to_trigger: bool,
    pub macro_cycle: bool,
    /// Cycle time in milliseconds.
    pub cycle_ms: u16,
    pub steps: Vec<MacroStep>,
}

/// One macro step. Step 29 has no delay on the wire, so `delay_ms` is 0 there.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroStep {
    pub key: u8,
    pub hold_ms: u16,
    pub delay_ms: u16,
}

/// Dead-zone module: enable flag plus four bounds (front, back, anti-front, anti-back).
/// Cyclone 2 stores one byte each (0..=100); the V2 family stores u16 big-endian (0..=1000).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadZone {
    pub enabled: bool,
    pub front: u16,
    pub back: u16,
    pub anti_front: u16,
    pub anti_back: u16,
}

/// Curve module: 13 bytes, five (original, target) points. Section 8.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linear {
    pub enabled: bool,
    pub curve_type: u8,
    pub curvature: u8,
    pub points: [(u8, u8); 5],
}

/// Trigger packet (L2/R2, 28 bytes). Section 5.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub turbo: u8,
    pub dead: DeadZone,
    pub mapped: bool,
    pub map: [u8; 3],
    pub toggle_on_press: bool,
    /// Raw quick-trigger status byte: bit7 enabled, bit1 mode 2.
    pub quick_trigger_status: u8,
    pub quick_trigger_start: u8,
    pub quick_trigger_end: u8,
    pub linear: Linear,
}

/// Map module, 12 bytes, shared by sticks and motion. `map_index` is an output-type enum (Left 1, Right 2, Wheel 3, Mouse 4).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapModule {
    pub enabled: bool,
    pub x_flip: bool,
    pub y_flip: bool,
    /// V2 motion only.
    #[serde(default)]
    pub z_flip: bool,
    pub axis_ratio: u8,
    pub mouse_dpi: u8,
    pub map_index: u8,
    pub map_cross: u8,
    pub map_up: u8,
    pub map_down: u8,
    pub map_left: u8,
    pub map_right: u8,
    pub map_dead: u8,
}

/// Left/Right stick packet (32 bytes). Section 6.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stick {
    pub enabled: bool,
    pub square: bool,
    pub dead: DeadZone,
    pub linear: Linear,
    pub map: MapModule,
}

/// Aim/Tilt motion packet (33 bytes, V2 41 bytes). Section 7. `active_axis` is the raw bit field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Motion {
    pub sensor_profile_status: u8,
    pub sensor_quick_key_value: u8,
    /// V2 only: second and third quick keys.
    #[serde(default)]
    pub quick_key_value2: u8,
    #[serde(default)]
    pub quick_key_value3: u8,
    pub active_axis: u8,
    pub dead: DeadZone,
    pub linear: Linear,
    pub map: MapModule,
    /// V2 only.
    #[serde(default)]
    pub stick_sensitivity: u8,
}

/// Fun block (32 bytes at offset 32). Section 2. Motors are levels 0-4.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fun {
    pub motors: [u8; 4],
    pub audio_enabled: u8,
    pub audio_volume: u8,
    pub audio_mixer: u8,
    pub mic_mute: u8,
    pub mic_sensitivity: u8,
    pub shift_enabled: u8,
    pub shift_value: u8,
    pub dpad_diagonal_lock: u8,
    pub xinput_abxy_change: u8,
    pub switch_abxy_change: u8,
    pub report_rate_gears: u8,
    /// V2 only.
    #[serde(default)]
    pub sensor_quick_mode: u8,
    #[serde(default)]
    pub stick_resolution: u8,
    /// G7ProCE and K2_8K only.
    #[serde(default)]
    pub ls_anti_jitter: u8,
    #[serde(default)]
    pub rs_anti_jitter: u8,
}

/// T3CE touch key (Ext8, Ext9; 10 bytes): key fields plus a trigger mode and a 16-bit sensitivity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TouchKey {
    pub turbo: u8,
    pub mapped: bool,
    pub map: [u8; 3],
    pub toggle_on_press: bool,
    pub trigger_mode: u8,
    pub sensitivity: u16,
}

/// One 680-byte profile block. `raw` keeps the bytes the codec does not model
/// (reserved bytes), so encode writes them back unchanged. Not serialized.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub fun: Fun,
    /// Order: Up, Under, Left, Right, L1, R1, L3, R3, CRO, CIR, SQU, TRI, PS, Sel, Sta, Camera.
    pub buttons: [Button; 16],
    pub fl1: FunctionKey,
    pub fr1: FunctionKey,
    pub l2: Trigger,
    pub r2: Trigger,
    pub left_stick: Stick,
    pub right_stick: Stick,
    pub aim: Motion,
    pub tilt: Motion,
    /// Function keys after FL1/FR1, in the model's order (`models::V2Spec::fn_keys[2..]`).
    #[serde(default)]
    pub extra: Vec<FunctionKey>,
    /// T3CE touch keys Ext8, Ext9.
    #[serde(default)]
    pub touch: Vec<TouchKey>,
    #[serde(skip)]
    pub raw: Vec<u8>,
}

/// V2 effect light (slot 32): five animations of a mode, per-model settings and HSV zones.
/// Setting and zone names: `models::V2Spec::light_settings` / `light_zones`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectLight {
    pub animations: Vec<EffectAnimation>,
    pub raise_wake_up: bool,
    /// Tail bytes after RaiseWakeUp, named by `models::V2Spec::light_tail`.
    pub tail: Vec<u8>,
    #[serde(skip)]
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectAnimation {
    pub mode: u8,
    pub settings: Vec<u8>,
    pub zones: Vec<Hsv>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hsv {
    pub h: u16,
    pub s: u8,
    pub v: u8,
}

/// Light profile (slot 32, 635 bytes). Layout: `docs/protocol/cyclone2-light.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightProfile {
    /// Active animation, 0..=3.
    pub config_index: u8,
    /// Always 5 animations.
    pub animations: Vec<LightAnimation>,
    pub audio_reactive: bool,
    pub user_effect_index: u8,
    pub profile_led: [u8; 3],
    pub raise_wake_up: bool,
    pub standby_time: u8,
    #[serde(skip)]
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimation {
    /// Active frame count; inactive frame slots are still stored on the device.
    pub key_frame_count: u8,
    pub effect_count: u8,
    /// User-facing speed (the wire stores `20 - speed`).
    pub speed: u8,
    /// 0..=100.
    pub brightness: u8,
    /// Up to 8 key frames, each 5 RGB zones.
    pub frames: Vec<[[u8; 3]; 5]>,
}

/// Decoded input report 0x12 (`docs/protocol/cyclone2-input.md`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepadState {
    pub id: String,
    /// Pressed button names (keycode names, e.g. "CRO", "Up", "FL1").
    pub buttons: Vec<String>,
    /// Sticks in -1.0..=1.0.
    pub lx: f32,
    pub ly: f32,
    pub rx: f32,
    pub ry: f32,
    /// Triggers 0..=255.
    pub lt: u8,
    pub rt: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceMode {
    Wired,
    Wireless,
    WiredAdc,
    WirelessAdc,
    /// VID 0x3537 but not a config PID: nothing is ever sent to it.
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    /// HID path of the config collection (or of the first collection when unsupported).
    pub id: String,
    /// "C2" for Cyclone 2.
    pub model: String,
    pub name: String,
    pub vid: u16,
    pub pid: u16,
    pub mode: DeviceMode,
    pub config_capable: bool,
    /// USB serial number, when the device reports one. Two entries with the same serial are one controller.
    pub serial: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub id: String,
    /// Model id from `models::MODELS`, e.g. "C2", "G7ProCE".
    #[serde(default)]
    pub model: String,
    /// Device firmware, e.g. "3.4.0".
    pub firmware: String,
    /// Dongle firmware; `None` when the reply carries no digits.
    pub dongle_firmware: Option<String>,
    /// 1..=4.
    pub current_profile: u8,
    /// Profiles 1..=4 in order.
    pub profiles: Vec<Profile>,
    /// Shift profile (slot 5), same codec as `profiles`.
    pub shift: Option<Profile>,
    /// Cyclone 2 light.
    pub light: Option<LightProfile>,
    /// V2 family light.
    #[serde(default)]
    pub effect_light: Option<EffectLight>,
    pub battery: Option<u8>,
    pub charging: Option<bool>,
    /// False for models that are read-only in OpenSir until hardware-tested (every V2 family model).
    pub writable: bool,
}
