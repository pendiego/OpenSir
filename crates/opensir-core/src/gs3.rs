//! "gs3" config protocol of the SAPJoy-family controllers (T3XS; the T4N2 library sends the same
//! frames to any VID 0x3537 usage-0xFF7A interface): `docs/protocol/sapjoy-t3xs-n2s.md` section 2.
//! Frame: `00 A5 len C0 cmd payload.. cs`, len = payload + 5, cs = sum of bytes 1.. (u8).
//! Payloads are the packed structs of `docs/protocol/sapjoy-gs3.md`; vectors produced by running
//! the vendor DLL: `docs/protocol/sapjoy-gs3-vectors.json`.

/// Command ids (`GS_CMD`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmd {
    GetDeviceInfo = 1,
    GetMacAddress = 2,
    GetVersion = 3,
    SetCustomCalibration = 4,
    GetCustomCalibration = 5,
    SetDPadParam = 6,
    GetDPadParam = 7,
    SetAbxyRotationTest = 8,
    SetKeyMappingHsv = 9,
    GetKeyMappingHsv = 10,
    SetLightColorTest = 11,
    SetLightParam = 12,
    GetLightParam = 13,
    SetLightColorHsv = 14,
    GetLightColorHsv = 15,
    RecordMacro = 16,
    SetMacroStep = 17,
    GetMacroStep = 18,
    SetStickCurveHsv = 19,
    GetStickCurveHsv = 20,
    SetStickMapping = 21,
    GetStickMapping = 22,
    SetTriggerParam = 23,
    GetTriggerParam = 24,
    SetVibrateTest = 25,
    SetVibrateLevel = 26,
    GetVibrateLevel = 27,
    SetGamepadStateReport = 28,
    SetMotionSenseCurveHsv = 29,
    GetMotionSenseCurveHsv = 30,
    SetMotionSenseMapping = 31,
    GetMotionSenseMapping = 32,
    SetConfigPage = 33,
    GetConfigPage = 34,
    SaveConfigPage = 35,
    ResetConfigPage = 36,
    SetDeviceType = 37,
    GetDeviceType = 38,
    SetGamepadParam = 45,
    GetGamepadParam = 46,
    SetReportRate = 47,
    GetReportRate = 48,
}

pub const HEADER: u8 = 0xA5;
pub const REQUEST: u8 = 0xC0;
pub const REPLY: u8 = 0xC1;

/// Request frame (report byte 0 included). The writer pads it to the output report size.
pub fn frame(cmd: Cmd, payload: &[u8]) -> Vec<u8> {
    let mut f = Vec::with_capacity(payload.len() + 6);
    f.extend_from_slice(&[0x00, HEADER, (payload.len() + 5) as u8, REQUEST, cmd as u8]);
    f.extend_from_slice(payload);
    f.push(f[1..].iter().fold(0u8, |s, b| s.wrapping_add(*b)));
    f
}

/// Payload of a reply to `cmd`: `A5 len C1 cmd ..`, payload from byte 5 (byte 4 = status, inferred).
pub fn reply_payload(cmd: Cmd, r: &[u8]) -> Option<&[u8]> {
    (r.len() > 5 && r[0] == HEADER && r[2] == REPLY && r[3] == cmd as u8).then(|| &r[5..])
}

/// Packed little-endian HSV colour (h u16, s, v).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hsv {
    pub h: u16,
    pub s: u8,
    pub v: u8,
}

impl Hsv {
    fn put(&self, o: &mut Vec<u8>) {
        o.extend_from_slice(&self.h.to_le_bytes());
        o.extend_from_slice(&[self.s, self.v]);
    }
    fn get(b: &[u8]) -> Self {
        Self { h: u16::from_le_bytes([b[0], b[1]]), s: b[2], v: b[3] }
    }
}

/// Key mapping (`SAPJOYUNI_KEYMAPPING_HSV`, 17 bytes): `code` holds the keyboard / mouse /
/// joystick / media union (4 bytes) chosen by `mapping_type`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyMapping {
    pub page: u8,
    pub key: u8,
    pub trigger: u8,
    pub color: Hsv,
    pub mapping_key: u8,
    pub toggle: u8,
    pub turbo_mode: u8,
    pub turbo_times: u8,
    pub mapping_type: u8,
    pub gamepad_key_count: u8,
    pub code: [u8; 4],
}

impl KeyMapping {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.page, self.key, self.trigger];
        self.color.put(&mut o);
        o.extend_from_slice(&[self.mapping_key, self.toggle, self.turbo_mode, self.turbo_times, self.mapping_type, self.gamepad_key_count]);
        o.extend_from_slice(&self.code);
        o
    }
    pub fn decode(b: &[u8]) -> Self {
        Self {
            page: b[0], key: b[1], trigger: b[2], color: Hsv::get(&b[3..]), mapping_key: b[7], toggle: b[8],
            turbo_mode: b[9], turbo_times: b[10], mapping_type: b[11], gamepad_key_count: b[12], code: [b[13], b[14], b[15], b[16]],
        }
    }
}

/// Stick curve (`SAPJOYUNI_STICKCURVE_HSV`, 22 bytes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StickCurve {
    pub stick: u8,
    pub page: u8,
    pub key: u8,
    pub trigger: u8,
    pub color: Hsv,
    pub kind: u8,
    pub ratio: u8,
    pub points: [(u8, u8); 5],
    pub curve_type: u8,
    pub curvature: u8,
}

impl StickCurve {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.stick, self.page, self.key, self.trigger];
        self.color.put(&mut o);
        o.extend_from_slice(&[self.kind, self.ratio]);
        self.points.iter().for_each(|&(x, y)| o.extend_from_slice(&[x, y]));
        o.extend_from_slice(&[self.curve_type, self.curvature]);
        o
    }
    pub fn decode(b: &[u8]) -> Self {
        Self {
            stick: b[0], page: b[1], key: b[2], trigger: b[3], color: Hsv::get(&b[4..]), kind: b[8], ratio: b[9],
            points: std::array::from_fn(|i| (b[10 + 2 * i], b[11 + 2 * i])), curve_type: b[20], curvature: b[21],
        }
    }
}

/// Motion curve (`SAPJOYUNI_MOTIONSENSECURVE_HSV`, 20 bytes): a stick curve without `stick` and `type`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MotionCurve {
    pub page: u8,
    pub key: u8,
    pub trigger: u8,
    pub color: Hsv,
    pub ratio: u8,
    pub points: [(u8, u8); 5],
    pub curve_type: u8,
    pub curvature: u8,
}

impl MotionCurve {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.page, self.key, self.trigger];
        self.color.put(&mut o);
        o.push(self.ratio);
        self.points.iter().for_each(|&(x, y)| o.extend_from_slice(&[x, y]));
        o.extend_from_slice(&[self.curve_type, self.curvature]);
        o
    }
    pub fn decode(b: &[u8]) -> Self {
        Self {
            page: b[0], key: b[1], trigger: b[2], color: Hsv::get(&b[3..]), ratio: b[7],
            points: std::array::from_fn(|i| (b[8 + 2 * i], b[9 + 2 * i])), curve_type: b[18], curvature: b[19],
        }
    }
}

/// Trigger (`SAPJOYUNI_TRIGGERPARAM`, 16 bytes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TriggerParam {
    pub trigger: u8,
    pub kind: u8,
    pub trigger_value: u8,
    pub release_value: u8,
    pub points: [(u8, u8); 5],
    pub curve_type: u8,
    pub curvature: u8,
}

impl TriggerParam {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.trigger, self.kind, self.trigger_value, self.release_value];
        self.points.iter().for_each(|&(x, y)| o.extend_from_slice(&[x, y]));
        o.extend_from_slice(&[self.curve_type, self.curvature]);
        o
    }
    pub fn decode(b: &[u8]) -> Self {
        Self {
            trigger: b[0], kind: b[1], trigger_value: b[2], release_value: b[3],
            points: std::array::from_fn(|i| (b[4 + 2 * i], b[5 + 2 * i])), curve_type: b[14], curvature: b[15],
        }
    }
}

/// Light colour (`SAPJOYUNI_LIGHTCOLOR_HSV`, 17 bytes): one light name, four LEDs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LightColor {
    pub name: u8,
    pub leds: [Hsv; 4],
}

impl LightColor {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.name];
        self.leds.iter().for_each(|l| l.put(&mut o));
        o
    }
    pub fn decode(b: &[u8]) -> Self {
        Self { name: b[0], leds: std::array::from_fn(|i| Hsv::get(&b[1 + 4 * i..])) }
    }
}

/// Macro step (`SAPJOYUNI_MACROSTEP`, 16 bytes). Times are in 10 ms units, big-endian on the wire.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroStep {
    pub key: u8,
    pub index: u8,
    pub trigger: u8,
    pub cycle: u8,
    pub loop_time: u16,
    pub mapping_type: u8,
    pub gamepad_key_count: u8,
    pub code: [u8; 4],
    pub start_time: u16,
    pub end_time: u16,
}

impl MacroStep {
    pub fn encode(&self) -> Vec<u8> {
        let mut o = vec![self.key, self.index, self.trigger, self.cycle];
        o.extend_from_slice(&self.loop_time.to_be_bytes());
        o.extend_from_slice(&[self.mapping_type, self.gamepad_key_count]);
        o.extend_from_slice(&self.code);
        o.extend_from_slice(&self.start_time.to_be_bytes());
        o.extend_from_slice(&self.end_time.to_be_bytes());
        o
    }
    pub fn decode(b: &[u8]) -> Self {
        Self {
            key: b[0], index: b[1], trigger: b[2], cycle: b[3], loop_time: u16::from_be_bytes([b[4], b[5]]),
            mapping_type: b[6], gamepad_key_count: b[7], code: [b[8], b[9], b[10], b[11]],
            start_time: u16::from_be_bytes([b[12], b[13]]), end_time: u16::from_be_bytes([b[14], b[15]]),
        }
    }
}

/// Requests whose payload is a plain byte struct (fields in header order, all u8):
/// stick mapping (12: stick, angle, inner/outer trigger range, stick dir x/y, mode, state,
/// mouse sensitivity x/y, mouse dir x/y), vibrate level (6), light param (3: brightness, speed,
/// mode), d-pad (4: type, swap mode, layout, smart start/stop), vibrate test (4), motion mapping
/// (11), gamepad param (9: sleep minutes u16 LE, auto sleep, 6 reserved).
pub fn set_bytes(cmd: Cmd, fields: &[u8]) -> Vec<u8> {
    frame(cmd, fields)
}

/// Report rate request: u16 little-endian (e.g. 1000, 8000).
pub fn set_report_rate(hz: u16) -> Vec<u8> {
    frame(Cmd::SetReportRate, &hz.to_le_bytes())
}

/// Gamepad state reports: 0 off, 1 original, 2 mapped.
pub fn set_gamepad_state_report(mode: u8) -> Vec<u8> {
    frame(Cmd::SetGamepadStateReport, &[mode])
}
