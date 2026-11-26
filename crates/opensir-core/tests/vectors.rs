//! Replays `docs/protocol/*-vectors.json` against the codec and command builders.
use opensir_core::command::*;
use opensir_core::profile::{decode, encode};
use opensir_core::{keycodes::key_name, model::*};
use serde_json::json;
use serde_json::Value;

const PROFILES: &str = include_str!("../../../docs/protocol/cyclone2-profile-vectors.json");
const COMMANDS: &str = include_str!("../../../docs/protocol/cyclone2-command-vectors.json");

fn parse_hex(s: &str) -> Vec<u8> {
    let c: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    (0..c.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&c[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn profile_vectors_decode_and_roundtrip() {
    let v: Value = serde_json::from_str(PROFILES).unwrap();
    let entries = v.as_array().unwrap();
    let (mut decoded, mut exact, mut semantic) = (0, 0, 0);
    for e in entries {
        let name = e["name"].as_str().unwrap();
        let bytes = parse_hex(e["hex"].as_str().unwrap());
        assert_eq!(bytes.len() as u64, e["length"].as_u64().unwrap(), "{name}");
        let p = decode(&bytes).unwrap_or_else(|err| panic!("{name}: {err}"));
        decoded += 1;
        if let Some(n) = e["input_json"]["ProfileName"].as_str() {
            assert_eq!(p.name, n, "{name}");
        }
        if e["source"] == "vendor-exec" {
            compare_vendor(&p, &e["decoded_json"], name);
            semantic += 1;
        }
        if e["roundtrip"].as_bool() == Some(true) {
            assert_eq!(encode(&p).unwrap(), bytes, "{name}");
            exact += 1;
        }
    }
    assert_eq!(decoded, 30);
    assert_eq!(exact, 22);
    assert_eq!(semantic, 9);
}

fn compare_vendor(p: &Profile, want: &Value, name: &str) {
    assert_eq!(want["ProfileName"], p.name, "{name}");
    let [left, right, lt, rt] = p.fun.motors;
    assert_eq!(want["BasicInfo"]["motor"], json!({
        "leftMotor": left, "rightMotor": right, "ltMotor": lt, "rtMotor": rt,
    }), "{name}: motors");
    let keys = &want["KeyProfiles"];
    let mapping = |slot: &str, turbo: u8, mapped: bool, map: [u8; 3], toggle: bool| {
        let v = &keys[slot]["key"];
        let names: Vec<_> = map.into_iter().filter(|&code| mapped && code != 0).map(key_name).collect();
        assert_eq!(v["turbo"], turbo, "{name}: {slot}.turbo");
        assert_eq!(v["mapKeys"], json!(names), "{name}: {slot}.map");
        assert_eq!(v["toggleOnPress"], toggle, "{name}: {slot}.toggle");
    };
    let slots = ["Up", "Under", "Left", "Right", "L1", "R1", "L3", "R3",
        "CRO", "CIR", "SQU", "TRI", "PS", "Sel", "Sta", "Camera"];
    for (slot, b) in slots.into_iter().zip(&p.buttons) {
        mapping(slot, b.turbo, b.mapped, b.map, b.toggle_on_press);
    }
    for (slot, f) in [("FL1", &p.fl1), ("FR1", &p.fr1)] {
        mapping(slot, f.turbo, f.mapped, f.map, f.toggle_on_press);
        let steps: Vec<_> = f.steps.iter().map(|s| {
            json!({"key": key_name(s.key), "hold": s.hold_ms, "delay": s.delay_ms})
        }).collect();
        assert_eq!(keys[slot]["key"]["keyType"] == "Macro", f.macro_enabled, "{name}: {slot}.macro");
        assert_eq!(keys[slot]["macro"], json!({
            "cycle": f.macro_cycle, "cycleTime": f.cycle_ms,
            "isPressToTrigger": f.macro_press_to_trigger, "steps": steps,
        }), "{name}: {slot}.macro details");
    }
    for (slot, t) in [("L2", &p.l2), ("R2", &p.r2)] {
        mapping(slot, t.turbo, t.mapped, t.map, t.toggle_on_press);
        compare_curve(&t.dead, &t.linear, &keys[slot], name, slot);
        let enabled = t.quick_trigger_status & 128 != 0;
        let (start, end) = if enabled { (t.quick_trigger_start, t.quick_trigger_end) } else { (10, 90) };
        let mode = if t.quick_trigger_status & 2 != 0 { 2 } else { 1 };
        assert_eq!(keys[slot]["hairTrigger"], json!({
            "enabled": enabled, "mode": mode, "beginRange": start, "endRange": end,
        }), "{name}: {slot}.hairTrigger");
    }
    for (slot, s) in [("Left", &p.left_stick), ("Right", &p.right_stick)] {
        let v = &want["Sticks"][slot];
        assert_eq!(v["rawEnabled"], !s.enabled || s.square, "{name}: {slot}.rawEnabled");
        compare_curve(&s.dead, &s.linear, v, name, slot);
        compare_map(&s.map, v, name, slot);
    }
    for (slot, m) in [("Aim", &p.aim), ("Tilt", &p.tilt)] {
        let v = &want["Motion"][slot];
        assert_eq!(v["activeAxis"], m.active_axis, "{name}: {slot}.activeAxis");
        assert_eq!(v["activateBehavior"], m.sensor_profile_status, "{name}: {slot}.activateBehavior");
        assert_eq!(v["triggerKey"], key_name(m.sensor_quick_key_value), "{name}: {slot}.triggerKey");
        compare_curve(&m.dead, &m.linear, v, name, slot);
        compare_map(&m.map, v, name, slot);
    }
}

fn compare_curve(d: &DeadZone, l: &Linear, want: &Value, name: &str, slot: &str) {
    assert!(d.enabled && l.enabled, "{name}: {slot} modules not enabled");
    assert_eq!(want["deadzone"], json!({
        "beginDeadzone": d.front, "endDeadzone": d.back,
        "beginAntiDeadzone": d.anti_front, "endAntiDeadzone": d.anti_back,
    }), "{name}: {slot}.deadzone");
    let scale = |v: u8, front: u16, back: u16| {
        (f64::from(v) / 255.0 * (f64::from(back) - f64::from(front)) + f64::from(front)).round() as i64
    };
    let points: Vec<_> = l.points[1..4].iter().map(|&(x, y)| json!({
        "x": scale(x, d.front, d.back), "y": scale(y, d.anti_front, d.anti_back),
    })).collect();
    assert_eq!(want["curve"], json!({
        "type": l.curve_type, "curvature": l.curvature, "points": points,
    }), "{name}: {slot}.curve");
}

fn compare_map(m: &MapModule, want: &Value, name: &str, slot: &str) {
    assert!(m.enabled, "{name}: {slot} map not enabled");
    assert_eq!(want["advanceMap"], json!({
        "outputType": m.map_index, "reverseHorizontal": m.x_flip,
        "reverseVertical": m.y_flip, "cross": m.map_cross,
        "axisRatio": m.axis_ratio, "mouseDpi": m.mouse_dpi,
        "wheelMapping": {
            "outline": key_name(m.map_dead), "up": key_name(m.map_up),
            "under": key_name(m.map_down), "left": key_name(m.map_left), "right": key_name(m.map_right),
        }
    }), "{name}: {slot}.advanceMap");
}

#[test]
fn standalone_input_report_decodes_buttons_axes_and_battery() {
    use opensir_core::input::{battery, parse};
    let mut b = [0; 64];
    b[..10].copy_from_slice(&[0x12, 0, 128, 255, 124, 0x21, 0x11, 0x19, 34, 180]);
    b[35] = 1;
    b[36] = 75;
    b[60] = 0x30;
    let p = parse("device", &b).unwrap();
    assert_eq!(p.id, "device");
    assert_eq!((p.lx, p.ly, p.rx, p.ry, p.lt, p.rt), (-1.0, 0.0, 1.0, -0.0312, 34, 180));
    assert_eq!(p.buttons, ["Up", "Right", "CRO", "L1", "Sel", "PS", "FR1", "M"]);
    assert_eq!(battery(&b), Some((75, true)));
    assert!(parse("device", &b[..60]).is_none());
    b[0] = 0x0a;
    assert!(parse("device", &b).is_none());
    assert!(battery(&b).is_none());
}

#[test]
fn reset_is_scoped_and_sets_wire_defaults() {
    use opensir_core::reset::apply;
    let mut p = decode(&[0; 680]).unwrap();
    p.name = "Keep me".into();
    p.l2.turbo = 7;
    apply(&mut p, "Triggers").unwrap();
    assert_eq!(p.l2.turbo, 7);
    assert_eq!((p.l2.dead.front, p.l2.dead.back, p.l2.linear.points[1]), (5, 95, (57, 64)));
    assert_eq!(p.fun.motors, [0; 4]);
    apply(&mut p, "").unwrap();
    assert_eq!(p.name, "Keep me");
    assert_eq!(p.l2.turbo, 0);
    assert_eq!(p.fun.motors, [3; 4]);
    assert_eq!(p.aim.active_axis, 2);
    assert_eq!(p.tilt.map.map_index, 2);
    let previous = p.clone();
    assert!(apply(&mut p, "light").is_err());
    assert_eq!(p, previous);
}

#[test]
fn profile_json_roundtrip_omits_raw_and_rejects_wrong_envelope() {
    use opensir_core::profile::{export_json, import_json};
    let path = std::env::temp_dir().join(format!("opensir-profile-{}.json", std::process::id()));
    let path = path.to_str().unwrap();
    let mut p = decode(&[0; 680]).unwrap();
    p.name = "Meu perfil".into();
    export_json(&p, path).unwrap();
    let mut file: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(file["model"], "C2");
    assert!(file["profile"].get("raw").is_none());
    p.raw.clear();
    assert_eq!(import_json(path).unwrap(), p);
    for (field, invalid) in [("version", json!(2)), ("model", json!("other"))] {
        let original = std::mem::replace(&mut file[field], invalid);
        std::fs::write(path, serde_json::to_vec(&file).unwrap()).unwrap();
        assert!(import_json(path).is_err(), "{field} must be validated");
        file[field] = original;
    }
    std::fs::remove_file(path).unwrap();
}

fn command_packets(vs: &[Value], name: &str) -> Vec<Vec<u8>> {
    let v = vs
        .iter()
        .find(|v| v["name"] == name)
        .unwrap_or_else(|| panic!("missing vector {name}"));
    v["packets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| parse_hex(p["hex"].as_str().unwrap()))
        .collect()
}

#[test]
fn command_vectors_match_builders() {
    let v: Value = serde_json::from_str(COMMANDS).unwrap();
    let vs = v["vectors"].as_array().unwrap();

    let singles = [
        ("EnterProfileConfig", enter_profile_config()),
        ("ExitProfileConfig(0)", exit_profile_config()),
        ("ReadDeviceInfo(1)", read_device_info_1()),
        ("SetLightStatus(1,3)", set_light_status()),
        ("HeartBeat(0)", heartbeat(false)),
    ];
    for (name, frame) in singles {
        let want = command_packets(vs, name);
        assert_eq!(want.len(), 1, "{name}");
        assert_eq!(&frame[..want[0].len()], &want[0][..], "{name}");
    }

    for (name, block, total) in [("ReadProfile(1) chunks", 1u8, 680usize), ("ReadProfile(32) chunks", 32, 635)] {
        let want = command_packets(vs, name);
        let mut got = Vec::new();
        let mut off = 0usize;
        while off < total {
            let n = (total - off).min(58);
            got.push(read_chunk(block, off as u16, n as u8));
            off += n;
        }
        assert_eq!(got.len(), want.len(), "{name}");
        for (g, w) in got.iter().zip(&want) {
            assert_eq!(&g[..w.len()], &w[..], "{name}");
        }
    }
}
