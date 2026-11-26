//! Profile block codec: bytes <-> `model::Profile`.
//! Cyclone 2 (680 B): `docs/protocol/cyclone2-profile.md`. V2 family (T3CE, G7ProCE, K2_8K):
//! same section order with 169-byte function keys, 16-bit dead zones and longer trigger, stick and
//! motion packets (`docs/protocol/layouts/*-profile.json`).
//! Encode starts from the bytes kept in `Profile::raw`, so reserved bytes survive.

use crate::model::*;
use crate::models::{Family, Model, V2Spec};

pub const PROFILE_LEN: usize = 680;
const NAME: usize = 0;
const FUN: usize = 32;
const BUTTONS: usize = 64;
const FUNCS: usize = 176;
const MAX_STEPS: usize = 30;

/// Section widths of one family.
struct Shape {
    /// V2: u16 dead zones, wider motion and map modules.
    wide: bool,
    func_len: usize,
    max_steps: usize,
    trigger_len: usize,
    stick_len: usize,
    motion_len: usize,
}

const C2: Shape = Shape { wide: false, func_len: 159, max_steps: MAX_STEPS, trigger_len: 28, stick_len: 32, motion_len: 33 };
const V2: Shape = Shape { wide: true, func_len: 169, max_steps: 32, trigger_len: 32, stick_len: 36, motion_len: 41 };
const TOUCH_LEN: usize = 10;

/// Offsets of one model's sections.
struct Layout {
    shape: &'static Shape,
    len: usize,
    funcs: usize,
    anti_jitter: bool,
    touch: bool,
}

impl Layout {
    fn of(model: &Model) -> Option<Self> {
        match model.family {
            Family::C2 => Some(Self { shape: &C2, len: PROFILE_LEN, funcs: 2, anti_jitter: false, touch: false }),
            Family::V2(s) => Some(Self::v2(s)),
            Family::Unsupported => None,
        }
    }
    fn v2(s: &V2Spec) -> Self {
        Self { shape: &V2, len: s.profile_len, funcs: s.fn_keys.len(), anti_jitter: s.anti_jitter, touch: s.touch_keys }
    }
    fn func(&self, i: usize) -> usize {
        FUNCS + i * self.shape.func_len
    }
    fn trigger(&self, i: usize) -> usize {
        self.func(self.funcs) + i * self.shape.trigger_len
    }
    fn stick(&self, i: usize) -> usize {
        self.trigger(2) + i * self.shape.stick_len
    }
    fn motion(&self, i: usize) -> usize {
        self.stick(2) + i * self.shape.motion_len
    }
    fn touch_key(&self, i: usize) -> usize {
        self.motion(2) + i * TOUCH_LEN
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    BadLength(usize),
    TooManySteps(usize),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLength(n) => write!(f, "profile block has an unexpected length: {n} bytes"),
            Self::TooManySteps(n) => write!(f, "macro has {n} steps, more than the controller stores"),
        }
    }
}

impl std::error::Error for ProfileError {}

const JSON_FORMAT: &str = "opensir-profile";
const JSON_VERSION: u8 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct ProfileFile<S, P> {
    format: S,
    version: u8,
    model: S,
    profile: P,
}

/// Cyclone 2 profile file.
pub fn export_json(profile: &Profile, path: &str) -> std::io::Result<()> {
    export_json_for("C2", profile, path)
}

pub fn import_json(path: &str) -> std::io::Result<Profile> {
    import_json_for("C2", path)
}

/// Profile file tagged with the model id; import refuses another model's file.
pub fn export_json_for(model: &str, profile: &Profile, path: &str) -> std::io::Result<()> {
    let file = ProfileFile { format: JSON_FORMAT, version: JSON_VERSION, model, profile };
    let bytes = serde_json::to_vec_pretty(&file).map_err(std::io::Error::other)?;
    std::fs::write(path, bytes)
}

pub fn import_json_for(model: &str, path: &str) -> std::io::Result<Profile> {
    use std::io::{Error, ErrorKind};

    let file: ProfileFile<String, Profile> = serde_json::from_slice(&std::fs::read(path)?)
        .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
    if file.format != JSON_FORMAT || file.version != JSON_VERSION || file.model != model {
        return Err(Error::new(ErrorKind::InvalidData, "unsupported OpenSir profile format, version, or model"));
    }
    let profile = file.profile;
    if profile.fun.motors.iter().any(|&level| level > 4) {
        return Err(Error::new(ErrorKind::InvalidData, "motor levels must be in 0..=4"));
    }
    let max_steps = if model == "C2" { MAX_STEPS } else { V2.max_steps };
    for key in [&profile.fl1, &profile.fr1].into_iter().chain(&profile.extra) {
        if key.steps.len() > max_steps {
            return Err(Error::new(ErrorKind::InvalidData, ProfileError::TooManySteps(key.steps.len())));
        }
    }
    Ok(profile)
}

/// Values the vendor app never writes (unknown key codes, out-of-range enums and dead zones), one line each.
/// `Session::save_profile` refuses an edit that adds one, so IPC callers and imported files cannot put them
/// in flash; values already on the device are left alone.
pub fn violations_for(model: &Model, p: &Profile) -> Vec<String> {
    let wide = Layout::of(model).is_some_and(|l| l.shape.wide);
    let dead_max = if wide { 1000 } else { 100 };
    let mut out = Vec::new();
    let mut key = |field: String, c: u8| {
        if c != 0 && crate::keycodes::key_name(c).is_empty() {
            out.push(format!("{field}: unknown key code {c}"));
        }
    };
    for (i, b) in p.buttons.iter().enumerate() {
        b.map.iter().for_each(|&c| key(format!("buttons[{i}].map"), c));
    }
    let funcs = [("fl1", &p.fl1), ("fr1", &p.fr1)].into_iter().chain(p.extra.iter().map(|k| ("extra", k)));
    for (name, f) in funcs {
        f.map.iter().for_each(|&c| key(format!("{name}.map"), c));
        f.steps.iter().for_each(|s| key(format!("{name}.steps"), s.key));
    }
    for (i, t) in p.touch.iter().enumerate() {
        t.map.iter().for_each(|&c| key(format!("touch[{i}].map"), c));
    }
    for (name, t) in [("l2", &p.l2), ("r2", &p.r2)] {
        t.map.iter().for_each(|&c| key(format!("{name}.map"), c));
    }
    let maps = [("leftStick", &p.left_stick.map), ("rightStick", &p.right_stick.map), ("aim", &p.aim.map), ("tilt", &p.tilt.map)];
    for (name, m) in maps {
        for c in [m.map_up, m.map_down, m.map_left, m.map_right] {
            key(format!("{name}.map"), c);
        }
    }
    let mut check = |ok: bool, what: String| {
        if !ok {
            out.push(what);
        }
    };
    check(p.fun.motors.iter().all(|&m| m <= 4), format!("fun.motors {:?} (0..=4)", p.fun.motors));
    check(p.fun.report_rate_gears <= 2, format!("fun.reportRateGears {} (0..=2)", p.fun.report_rate_gears));
    let deads = [("l2", &p.l2.dead), ("r2", &p.r2.dead), ("leftStick", &p.left_stick.dead), ("rightStick", &p.right_stick.dead), ("aim", &p.aim.dead), ("tilt", &p.tilt.dead)];
    for (name, d) in deads {
        let v = [d.front, d.back, d.anti_front, d.anti_back];
        check(v.iter().all(|&x| x <= dead_max), format!("{name}.dead {v:?} (0..={dead_max})"));
    }
    let linears = [("l2", &p.l2.linear), ("r2", &p.r2.linear), ("leftStick", &p.left_stick.linear), ("rightStick", &p.right_stick.linear), ("aim", &p.aim.linear), ("tilt", &p.tilt.linear)];
    for (name, l) in linears {
        check(l.curve_type <= 3, format!("{name}.curveType {} (0..=3)", l.curve_type));
    }
    for (name, t) in [("l2", &p.l2), ("r2", &p.r2)] {
        let (s, e) = (t.quick_trigger_start, t.quick_trigger_end);
        check(s <= 100 && e <= 100, format!("{name}.quickTrigger {s}..{e} (0..=100)"));
    }
    for (name, m) in maps {
        check(m.map_index <= 4, format!("{name}.mapIndex {} (0..=4)", m.map_index));
    }
    out
}

pub fn decode(b: &[u8]) -> Result<Profile, ProfileError> {
    decode_with(&Layout { shape: &C2, len: PROFILE_LEN, funcs: 2, anti_jitter: false, touch: false }, b)
}

pub fn encode(p: &Profile) -> Result<Vec<u8>, ProfileError> {
    encode_with(&Layout { shape: &C2, len: PROFILE_LEN, funcs: 2, anti_jitter: false, touch: false }, p)
}

/// Profile codec of any configurable model. Unsupported models report `BadLength(0)`.
pub fn decode_for(model: &Model, b: &[u8]) -> Result<Profile, ProfileError> {
    decode_with(&Layout::of(model).ok_or(ProfileError::BadLength(0))?, b)
}

pub fn encode_for(model: &Model, p: &Profile) -> Result<Vec<u8>, ProfileError> {
    encode_with(&Layout::of(model).ok_or(ProfileError::BadLength(0))?, p)
}

/// Profile block length of a configurable model.
pub fn len_for(model: &Model) -> Option<usize> {
    Layout::of(model).map(|l| l.len)
}

fn decode_with(l: &Layout, b: &[u8]) -> Result<Profile, ProfileError> {
    if b.len() != l.len {
        return Err(ProfileError::BadLength(b.len()));
    }
    let s = l.shape;
    let name = String::from_utf8_lossy(&b[NAME..NAME + 32]).trim_matches(|c: char| c == '\0' || c.is_whitespace()).to_string();
    let f = &b[FUN..FUN + 32];
    let (jl, jr) = if l.anti_jitter { (f[17], f[18]) } else { (0, 0) };
    let func = |i: usize| function_key(&b[l.func(i)..l.func(i) + s.func_len], s.max_steps);
    let trig = |i: usize| trigger(&b[l.trigger(i)..l.trigger(i) + s.trigger_len], s.wide);
    let stk = |i: usize| stick(&b[l.stick(i)..l.stick(i) + s.stick_len], s.wide);
    let mot = |i: usize| motion(&b[l.motion(i)..l.motion(i) + s.motion_len], s.wide);
    Ok(Profile {
        name,
        fun: Fun {
            motors: [motor_level(f[0]), motor_level(f[1]), motor_level(f[2]), motor_level(f[3])],
            audio_enabled: f[4],
            audio_volume: f[5],
            audio_mixer: f[6],
            mic_mute: f[7],
            mic_sensitivity: f[8],
            shift_enabled: f[9],
            shift_value: f[10],
            dpad_diagonal_lock: f[11],
            xinput_abxy_change: f[12],
            switch_abxy_change: f[13],
            report_rate_gears: f[14],
            sensor_quick_mode: if s.wide { f[15] } else { 0 },
            stick_resolution: if s.wide { f[16] } else { 0 },
            ls_anti_jitter: jl,
            rs_anti_jitter: jr,
        },
        buttons: std::array::from_fn(|i| button(&b[BUTTONS + 7 * i..])),
        fl1: func(0),
        fr1: func(1),
        l2: trig(0),
        r2: trig(1),
        left_stick: stk(0),
        right_stick: stk(1),
        aim: mot(0),
        tilt: mot(1),
        extra: (2..l.funcs).map(func).collect(),
        touch: if l.touch { (0..2).map(|i| touch_key(&b[l.touch_key(i)..l.touch_key(i) + TOUCH_LEN])).collect() } else { Vec::new() },
        raw: b.to_vec(),
    })
}

fn encode_with(l: &Layout, p: &Profile) -> Result<Vec<u8>, ProfileError> {
    let s = l.shape;
    let mut o = if p.raw.len() == l.len { p.raw.clone() } else { vec![0; l.len] };
    o[NAME..NAME + 32].fill(0);
    let nb = p.name.as_bytes();
    let mut n = nb.len().min(32);
    while !p.name.is_char_boundary(n) {
        n -= 1;
    }
    o[NAME..NAME + n].copy_from_slice(&nb[..n]);

    let f = &mut o[FUN..FUN + 32];
    for (i, m) in p.fun.motors.iter().enumerate() {
        f[i] = 25 * (*m).min(4);
    }
    f[4..15].copy_from_slice(&[
        p.fun.audio_enabled, p.fun.audio_volume, p.fun.audio_mixer, p.fun.mic_mute, p.fun.mic_sensitivity,
        p.fun.shift_enabled, p.fun.shift_value, p.fun.dpad_diagonal_lock, p.fun.xinput_abxy_change,
        p.fun.switch_abxy_change, p.fun.report_rate_gears,
    ]);
    if s.wide {
        f[15] = p.fun.sensor_quick_mode;
        f[16] = p.fun.stick_resolution;
    }
    if l.anti_jitter {
        f[17] = p.fun.ls_anti_jitter;
        f[18] = p.fun.rs_anti_jitter;
    }

    for (i, btn) in p.buttons.iter().enumerate() {
        put_button(&mut o[BUTTONS + 7 * i..BUTTONS + 7 * i + 7], btn);
    }
    for i in 0..l.funcs {
        let key = match i {
            0 => &p.fl1,
            1 => &p.fr1,
            _ => match p.extra.get(i - 2) {
                Some(k) => k,
                None => continue, // unsupplied key keeps its stored bytes
            },
        };
        put_function_key(&mut o[l.func(i)..l.func(i) + s.func_len], key, s.max_steps)?;
    }
    for (i, t) in [&p.l2, &p.r2].into_iter().enumerate() {
        put_trigger(&mut o[l.trigger(i)..l.trigger(i) + s.trigger_len], t, s.wide);
    }
    for (i, st) in [&p.left_stick, &p.right_stick].into_iter().enumerate() {
        put_stick(&mut o[l.stick(i)..l.stick(i) + s.stick_len], st, s.wide);
    }
    for (i, m) in [&p.aim, &p.tilt].into_iter().enumerate() {
        put_motion(&mut o[l.motion(i)..l.motion(i) + s.motion_len], m, s.wide);
    }
    if l.touch {
        for (i, t) in p.touch.iter().take(2).enumerate() {
            put_touch_key(&mut o[l.touch_key(i)..l.touch_key(i) + TOUCH_LEN], t);
        }
    }
    Ok(o)
}

/// Raw motor byte is `25 * level`; decode rounds and clamps to 0..=4.
fn motor_level(raw: u8) -> u8 {
    ((raw.min(100) as u32 + 12) / 25).min(4) as u8
}

fn turbo(en: u8, speed: u8) -> u8 {
    if en & 1 != 0 { speed } else { 0 }
}

fn put_turbo(out: &mut [u8], t: u8) {
    out[0] = (t > 0) as u8;
    out[1] = t;
}

fn button(b: &[u8]) -> Button {
    Button {
        turbo: turbo(b[0], b[1]),
        mapped: b[2] & 1 != 0,
        map: [b[3], b[4], b[5]],
        toggle_on_press: b[6] & 1 != 0,
    }
}

fn put_button(out: &mut [u8], v: &Button) {
    put_turbo(&mut out[0..2], v.turbo);
    out[2] = v.mapped as u8;
    out[3..6].copy_from_slice(&v.map);
    out[6] = v.toggle_on_press as u8;
}

/// Macro flag byte: bit0 enabled, bit1 press-to-trigger, bit2 cycle. The last step has no delay field.
fn function_key(b: &[u8], max_steps: usize) -> FunctionKey {
    let steps = (b[10] as usize).min(max_steps);
    FunctionKey {
        turbo: turbo(b[0], b[1]),
        mapped: b[2] & 1 != 0,
        map: [b[3], b[4], b[5]],
        toggle_on_press: b[6] & 1 != 0,
        macro_enabled: b[7] & 1 != 0,
        macro_press_to_trigger: b[7] & 2 != 0,
        macro_cycle: b[7] & 4 != 0,
        cycle_ms: u16::from_be_bytes([b[8], b[9]]),
        steps: (0..steps)
            .map(|i| {
                let o = 11 + 5 * i;
                let delay_ms = if i + 1 < max_steps { u16::from_be_bytes([b[o + 3], b[o + 4]]) } else { 0 };
                MacroStep { key: b[o], hold_ms: u16::from_be_bytes([b[o + 1], b[o + 2]]), delay_ms }
            })
            .collect(),
    }
}

fn put_function_key(out: &mut [u8], v: &FunctionKey, max_steps: usize) -> Result<(), ProfileError> {
    if v.steps.len() > max_steps {
        return Err(ProfileError::TooManySteps(v.steps.len()));
    }
    put_turbo(&mut out[0..2], v.turbo);
    out[2] = v.mapped as u8;
    out[3..6].copy_from_slice(&v.map);
    out[6] = v.toggle_on_press as u8;
    out[7] = v.macro_enabled as u8 | (v.macro_press_to_trigger as u8) << 1 | (v.macro_cycle as u8) << 2;
    out[8..10].copy_from_slice(&v.cycle_ms.to_be_bytes());
    out[10] = v.steps.len() as u8;
    for (i, s) in v.steps.iter().enumerate() {
        let o = 11 + 5 * i;
        out[o] = s.key;
        out[o + 1..o + 3].copy_from_slice(&s.hold_ms.to_be_bytes());
        if i + 1 < max_steps {
            out[o + 3..o + 5].copy_from_slice(&s.delay_ms.to_be_bytes());
        }
    }
    Ok(())
}

/// Dead-zone module length: enable byte plus four u8 (C2) or u16 big-endian (V2) bounds.
fn dead_len(wide: bool) -> usize {
    if wide { 9 } else { 5 }
}

fn dead(b: &[u8], wide: bool) -> DeadZone {
    let v = |i: usize| if wide { u16::from_be_bytes([b[1 + 2 * i], b[2 + 2 * i]]) } else { b[1 + i].into() };
    DeadZone { enabled: b[0] != 0, front: v(0), back: v(1), anti_front: v(2), anti_back: v(3) }
}

fn put_dead(out: &mut [u8], d: &DeadZone, wide: bool) {
    out[0] = d.enabled as u8;
    for (i, v) in [d.front, d.back, d.anti_front, d.anti_back].into_iter().enumerate() {
        if wide {
            out[1 + 2 * i..3 + 2 * i].copy_from_slice(&v.to_be_bytes());
        } else {
            out[1 + i] = v.min(255) as u8;
        }
    }
}

/// The Cyclone 2 encoder forces point 1 to (0,0) and point 5 to (255,255); the V2 encoder writes them as given.
fn linear(b: &[u8]) -> Linear {
    Linear {
        enabled: b[0] != 0,
        curve_type: b[1],
        curvature: b[2],
        points: std::array::from_fn(|i| (b[3 + 2 * i], b[4 + 2 * i])),
    }
}

fn put_linear(out: &mut [u8], l: &Linear, wide: bool) {
    out[0] = l.enabled as u8;
    out[1] = l.curve_type;
    out[2] = l.curvature;
    for i in 0..5 {
        let (o, t) = match i {
            0 if !wide => (0, 0),
            4 if !wide => (255, 255),
            _ => l.points[i],
        };
        out[3 + 2 * i] = o;
        out[4 + 2 * i] = t;
    }
}

/// Map module: 12 bytes; the V2 motion module has a z-flip after y-flip (13 bytes).
fn map_module(b: &[u8], z: bool) -> MapModule {
    let k = z as usize;
    MapModule {
        enabled: b[0] != 0,
        x_flip: b[1] != 0,
        y_flip: b[2] != 0,
        z_flip: z && b[3] != 0,
        axis_ratio: b[3 + k],
        mouse_dpi: b[4 + k],
        map_index: b[5 + k],
        map_cross: b[6 + k],
        map_up: b[7 + k],
        map_down: b[8 + k],
        map_left: b[9 + k],
        map_right: b[10 + k],
        map_dead: b[11 + k],
    }
}

fn put_map(out: &mut [u8], m: &MapModule, z: bool) {
    out[..3].copy_from_slice(&[m.enabled as u8, m.x_flip as u8, m.y_flip as u8]);
    let k = z as usize;
    if z {
        out[3] = m.z_flip as u8;
    }
    out[3 + k..12 + k].copy_from_slice(&[
        m.axis_ratio, m.mouse_dpi, m.map_index, m.map_cross, m.map_up, m.map_down, m.map_left, m.map_right, m.map_dead,
    ]);
}

fn trigger(b: &[u8], wide: bool) -> Trigger {
    let o = 2 + dead_len(wide);
    Trigger {
        turbo: turbo(b[0], b[1]),
        dead: dead(&b[2..], wide),
        mapped: b[o] != 0,
        map: [b[o + 1], b[o + 2], b[o + 3]],
        toggle_on_press: b[o + 4] & 1 != 0,
        quick_trigger_status: b[o + 5],
        quick_trigger_start: b[o + 6],
        quick_trigger_end: b[o + 7],
        linear: linear(&b[o + 8..o + 21]),
    }
}

fn put_trigger(out: &mut [u8], v: &Trigger, wide: bool) {
    let o = 2 + dead_len(wide);
    put_turbo(&mut out[0..2], v.turbo);
    put_dead(&mut out[2..o], &v.dead, wide);
    out[o] = v.mapped as u8;
    out[o + 1..o + 4].copy_from_slice(&v.map);
    out[o + 4] = v.toggle_on_press as u8;
    out[o + 5] = v.quick_trigger_status;
    out[o + 6] = v.quick_trigger_start;
    out[o + 7] = v.quick_trigger_end;
    put_linear(&mut out[o + 8..o + 21], &v.linear, wide);
}

fn stick(b: &[u8], wide: bool) -> Stick {
    let o = 2 + dead_len(wide);
    Stick {
        enabled: b[0] != 0,
        square: b[1] != 0,
        dead: dead(&b[2..], wide),
        linear: linear(&b[o..o + 13]),
        map: map_module(&b[o + 13..], false),
    }
}

fn put_stick(out: &mut [u8], v: &Stick, wide: bool) {
    let o = 2 + dead_len(wide);
    out[0] = v.enabled as u8;
    out[1] = v.square as u8;
    put_dead(&mut out[2..o], &v.dead, wide);
    put_linear(&mut out[o..o + 13], &v.linear, wide);
    put_map(&mut out[o + 13..], &v.map, false);
}

/// C2: status, quick key, active axis, dead, linear, map (33 B).
/// V2: status, quick keys 1-3, active axis, dead (u16), linear, map with z-flip, stick sensitivity (41 B).
fn motion(b: &[u8], wide: bool) -> Motion {
    let h = if wide { 5 } else { 3 };
    let o = h + dead_len(wide);
    Motion {
        sensor_profile_status: b[0],
        sensor_quick_key_value: b[1],
        quick_key_value2: if wide { b[2] } else { 0 },
        quick_key_value3: if wide { b[3] } else { 0 },
        active_axis: b[h - 1],
        dead: dead(&b[h..], wide),
        linear: linear(&b[o..o + 13]),
        map: map_module(&b[o + 13..], wide),
        stick_sensitivity: if wide { b[40] } else { 0 },
    }
}

fn put_motion(out: &mut [u8], v: &Motion, wide: bool) {
    let h = if wide { 5 } else { 3 };
    let o = h + dead_len(wide);
    out[0] = v.sensor_profile_status;
    out[1] = v.sensor_quick_key_value;
    if wide {
        out[2] = v.quick_key_value2;
        out[3] = v.quick_key_value3;
        out[40] = v.stick_sensitivity;
    }
    out[h - 1] = v.active_axis;
    put_dead(&mut out[h..o], &v.dead, wide);
    put_linear(&mut out[o..o + 13], &v.linear, wide);
    put_map(&mut out[o + 13..], &v.map, wide);
}

/// T3CE touch key: turbo (2), map enable + 3 values, toggle, trigger mode, sensitivity (u16 BE).
fn touch_key(b: &[u8]) -> TouchKey {
    TouchKey {
        turbo: turbo(b[0], b[1]),
        mapped: b[2] & 1 != 0,
        map: [b[3], b[4], b[5]],
        toggle_on_press: b[6] & 1 != 0,
        trigger_mode: b[7],
        sensitivity: u16::from_be_bytes([b[8], b[9]]),
    }
}

fn put_touch_key(out: &mut [u8], v: &TouchKey) {
    put_turbo(&mut out[0..2], v.turbo);
    out[2] = v.mapped as u8;
    out[3..6].copy_from_slice(&v.map);
    out[6] = v.toggle_on_press as u8;
    out[7] = v.trigger_mode;
    out[8..10].copy_from_slice(&v.sensitivity.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cyclone 2 section offsets (cyclone2-profile.md section 1).
    const FL1: usize = 176;
    const L2: usize = 494;
    const LSTICK: usize = 550;

    fn blank() -> Vec<u8> {
        vec![0; PROFILE_LEN]
    }

    #[test]
    fn default_profile_round_trips() {
        let p = Profile { raw: Vec::new(), ..Default::default() };
        let bytes = encode(&p).unwrap();
        assert_eq!(bytes.len(), PROFILE_LEN);
        let back = decode(&bytes).unwrap();
        assert_eq!(back.buttons, p.buttons);
        assert_eq!(encode(&back).unwrap(), bytes);
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(decode(&[0; 10]), Err(ProfileError::BadLength(10)));
    }

    #[test]
    fn button_vectors() {
        let mut b = blank();
        b[BUTTONS..BUTTONS + 7].copy_from_slice(&[0, 0, 1, 92, 0, 0, 0]);
        b[BUTTONS + 7..BUTTONS + 14].copy_from_slice(&[1, 5, 0, 0, 0, 0, 0]);
        let p = decode(&b).unwrap();
        assert!(p.buttons[0].mapped);
        assert_eq!(p.buttons[0].map, [92, 0, 0]);
        assert_eq!(p.buttons[1].turbo, 5);
    }

    #[test]
    fn left_stick_vector() {
        let bytes = [
            1, 0, 1, 15, 85, 0, 100, 1, 2, 80, 0, 0, 32, 14, 128, 99, 223, 241, 255, 255, 1, 1, 1, 60, 40, 3, 1, 92, 103,
            0, 0, 92,
        ];
        let mut b = blank();
        b[LSTICK..LSTICK + 32].copy_from_slice(&bytes);
        let s = decode(&b).unwrap().left_stick;
        assert!(s.enabled && s.dead.enabled);
        assert_eq!((s.dead.front, s.dead.back), (15, 85));
        assert_eq!(s.linear.points[1], (32, 14));
        assert_eq!(s.map.map_index, 3);
        assert_eq!((s.map.map_up, s.map.map_down, s.map.map_dead), (92, 103, 92));
    }

    #[test]
    fn l2_trigger_vector() {
        let bytes = [
            0, 0, 1, 10, 90, 5, 95, 0, 0, 0, 0, 0, 130, 20, 80, 1, 1, 50, 0, 0, 64, 43, 128, 128, 191, 213, 255, 255,
        ];
        let mut b = blank();
        b[L2..L2 + 28].copy_from_slice(&bytes);
        let t = decode(&b).unwrap().l2;
        assert_eq!(t.quick_trigger_status, 130);
        assert_eq!((t.quick_trigger_start, t.quick_trigger_end), (20, 80));
        assert_eq!(t.linear.points[1], (64, 43));
        let mut out = blank();
        put_trigger(&mut out[L2..L2 + 28], &t, false);
        assert_eq!(&out[L2..L2 + 28], &bytes);
    }

    #[test]
    fn fl1_macro_vector() {
        let mut b = blank();
        let f = FL1;
        b[f + 7] = 7;
        b[f + 8..f + 10].copy_from_slice(&[1, 44]);
        b[f + 10] = 2;
        b[f + 11..f + 16].copy_from_slice(&[92, 0, 50, 0, 20]);
        b[f + 16..f + 21].copy_from_slice(&[103, 0, 30, 0, 10]);
        let k = decode(&b).unwrap().fl1;
        assert!(k.macro_enabled && k.macro_press_to_trigger && k.macro_cycle);
        assert_eq!(k.cycle_ms, 300);
        assert_eq!(k.steps[0], MacroStep { key: 92, hold_ms: 50, delay_ms: 20 });
        assert_eq!(k.steps[1], MacroStep { key: 103, hold_ms: 30, delay_ms: 10 });
    }

    #[test]
    fn macro_round_trip_with_full_length() {
        let mut k = FunctionKey { macro_enabled: true, cycle_ms: 1234, ..Default::default() };
        k.steps = (0..30).map(|i| MacroStep { key: i as u8, hold_ms: 300 + i, delay_ms: if i < 29 { i } else { 0 } }).collect();
        let p = Profile { fl1: k.clone(), ..Default::default() };
        let back = decode(&encode(&p).unwrap()).unwrap();
        assert_eq!(back.fl1, k);
    }

    #[test]
    fn rejects_too_many_steps() {
        let k = FunctionKey { steps: vec![MacroStep::default(); 31], ..Default::default() };
        let p = Profile { fl1: k, ..Default::default() };
        assert_eq!(encode(&p), Err(ProfileError::TooManySteps(31)));
    }

    #[test]
    fn motors_round_trip() {
        assert_eq!(motor_level(50), 2);
        assert_eq!(motor_level(100), 4);
        assert_eq!(motor_level(0), 0);
    }
}
