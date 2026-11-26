//! Input report (report ID 0x12): Cyclone 2 (`docs/protocol/cyclone2-input.md`) and the V2 family
//! extras (`docs/protocol/hid-family-transport.md` section 6).

use crate::model::GamepadState;
use crate::models::{Family, Model};

pub const REPORT_ID: u8 = 0x12;

/// Model-aware parse: the C2 fields plus each V2 model's extra buttons.
pub fn parse_for(model: &Model, id: &str, report: &[u8]) -> Option<GamepadState> {
    let mut state = parse(id, report)?;
    let Family::V2(spec) = model.family else { return Some(state) };
    let push = |state: &mut GamepadState, flags: u8, names: &[&str]| {
        for (i, &name) in names.iter().enumerate() {
            if flags & (1 << i) != 0 {
                state.buttons.push(name.to_owned());
            }
        }
    };
    if spec.touch_keys {
        // T3CE: Ext1..Ext7 at byte 61 bits 0..6; triggers from bytes 62/63.
        push(&mut state, report.get(61).copied().unwrap_or(0), &spec.fn_keys[2..]);
        if report.len() > 63 {
            (state.lt, state.rt) = (report[62], report[63]);
        }
    } else if report.len() > 63 {
        // G7ProCE / K2_8K: FL2, FR2, FL3, FR3 at byte 63 bits 0..3.
        push(&mut state, report[63], &spec.fn_keys[2..]);
    }
    Some(state)
}

/// Battery offsets: T3CE charge 27 / level 28; C2, G7ProCE, K2_8K 35 / 36.
pub fn battery_for(model: &Model, report: &[u8]) -> Option<(u8, bool)> {
    match model.family {
        Family::V2(spec) if spec.touch_keys => {
            (report.len() > 28 && report[0] == REPORT_ID).then(|| (report[28].min(100), report[27] == 1))
        }
        _ => battery(report),
    }
}

pub fn parse(id: &str, report: &[u8]) -> Option<GamepadState> {
    if report.len() < 61 || report[0] != REPORT_ID {
        return None;
    }
    let mut buttons = Vec::new();
    let directions: &[&str] = match report[5] & 15 {
        0 => &["Up"],
        1 => &["Up", "Right"],
        2 => &["Right"],
        3 => &["Right", "Under"],
        4 => &["Under"],
        5 => &["Under", "Left"],
        6 => &["Left"],
        7 => &["Left", "Up"],
        _ => &[],
    };
    buttons.extend(directions.iter().copied().map(str::to_owned));
    let groups: [(u8, &[&str]); 4] = [
        (report[5] >> 4, &["SQU", "CRO", "CIR", "TRI"]),
        (report[6], &["L1", "R1", "L2", "R2", "Sel", "Sta", "L3", "R3"]),
        (report[7], &["PS", "Camera"]),
        (report[60] >> 3, &["FL1", "FR1", "M"]),
    ];
    for (flags, names) in groups {
        for (i, &name) in names.iter().enumerate() {
            if flags & (1 << i) != 0 {
                buttons.push(name.to_owned());
            }
        }
    }
    Some(GamepadState {
        id: id.to_owned(), buttons,
        lx: axis(report[1]), ly: axis(report[2]),
        rx: axis(report[3]), ry: axis(report[4]),
        lt: report[8], rt: report[9],
    })
}

fn axis(value: u8) -> f32 {
    let centered = f32::from(value) - 128.0;
    let max = if centered > 0.0 { 127.0 } else { 128.0 };
    // Match the device app's rounding: exact halves round toward positive infinity.
    (centered * 10_000.0 / max + 0.5).floor() / 10_000.0
}

pub fn battery(report: &[u8]) -> Option<(u8, bool)> {
    if report.len() < 37 || report[0] != REPORT_ID {
        return None;
    }
    // Offsets observed from the vendor app: report[35] = charge state (1 = charging), report[36] = level %.
    Some((report[36].min(100), report[35] == 1))
}
