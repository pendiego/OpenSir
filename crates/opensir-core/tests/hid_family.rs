//! Replays the V2 family vectors (`docs/protocol/{t3ce,g7proce,k2_8k}-raw-vectors.json`): the bytes
//! were produced by running the vendor codecs, `raw_decoded` is the vendor's own decode of them.
use opensir_core::model::*;
use opensir_core::models::{by_id, Family};
use opensir_core::{effect_light, profile};
use serde_json::Value;

const VECTORS: [(&str, &str); 3] = [
    ("T3CE", include_str!("../../../docs/protocol/t3ce-raw-vectors.json")),
    ("G7ProCE", include_str!("../../../docs/protocol/g7proce-raw-vectors.json")),
    ("K2_8K", include_str!("../../../docs/protocol/k2_8k-raw-vectors.json")),
];
const LAYOUTS: [(&str, &str); 3] = [
    ("T3CE", include_str!("../../../docs/protocol/layouts/t3ce-profile.json")),
    ("G7ProCE", include_str!("../../../docs/protocol/layouts/g7proce-profile.json")),
    ("K2_8K", include_str!("../../../docs/protocol/layouts/k2_8k-profile.json")),
];

fn hex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn n(v: &Value) -> u64 {
    v.as_u64().or(v.as_bool().map(u64::from)).unwrap_or_else(|| panic!("not a number: {v}"))
}

fn same_bytes(got: &[u8], want: &[u8], at: &str) {
    assert_eq!(got.len(), want.len(), "{at}: length");
    let diff: Vec<(usize, u8, u8)> = (0..got.len()).filter(|&i| got[i] != want[i]).map(|i| (i, got[i], want[i])).take(8).collect();
    assert!(diff.is_empty(), "{at}: (offset, ours, vendor) {diff:?}");
}

fn key(want: &Value, turbo: u8, mapped: bool, map: [u8; 3], toggle: bool, at: &str) {
    let t = &want["turbo_module"];
    assert_eq!(u64::from(turbo), if n(&t["turbo_en"]) & 1 == 1 { n(&t["turbo_speed"]) } else { 0 }, "{at}.turbo");
    let m = &want["map_module"];
    assert_eq!(mapped, n(&m["map_en"]) & 1 == 1, "{at}.mapped");
    assert_eq!(map.map(u64::from), [n(&m["map_value1"]), n(&m["map_value2"]), n(&m["map_value3"])], "{at}.map");
    assert_eq!(toggle, n(&want["toggle_en"]) & 1 == 1, "{at}.toggle");
}

fn dead(d: &DeadZone, w: &Value, at: &str) {
    let got = [u64::from(d.enabled), d.front.into(), d.back.into(), d.anti_front.into(), d.anti_back.into()];
    let want = ["dead_en", "front_dead", "back_dead", "anti_front_dead", "anti_back_dead"].map(|k| n(&w[k]));
    assert_eq!(got, want, "{at}.dead");
}

fn linear(l: &Linear, w: &Value, at: &str) {
    assert_eq!((u64::from(l.enabled), u64::from(l.curve_type), u64::from(l.curvature)),
        (n(&w["linear_module_en"]), n(&w["linear_status"]), n(&w["linear_data"])), "{at}.linear");
    for i in 0..5 {
        let (x, y) = l.points[i];
        assert_eq!((u64::from(x), u64::from(y)), (n(&w[format!("original_data{}", i + 1)]), n(&w[format!("target_data{}", i + 1)])), "{at}.point{i}");
    }
}

fn map(m: &MapModule, w: &Value, at: &str) {
    let got = [m.enabled as u64, m.x_flip as u64, m.y_flip as u64, m.axis_ratio.into(), m.mouse_dpi.into(), m.map_index.into(),
        m.map_cross.into(), m.map_up.into(), m.map_down.into(), m.map_left.into(), m.map_right.into(), m.map_dead.into()];
    let want = ["map_en", "x_flip", "y_flip", "axis_ratio", "mouse_dpi", "map_index", "map_cross", "map_up_value",
        "map_down_value", "map_left_value", "map_right_value", "map_dead_value"].map(|k| n(&w[k]));
    assert_eq!(got, want, "{at}.map");
    if let Some(z) = w.get("z_flip") {
        assert_eq!(m.z_flip as u64, n(z), "{at}.z_flip");
    }
}

fn compare(p: &Profile, w: &Value, fn_keys: &[&str], at: &str) {
    assert_eq!(p.name, w["ProfileName"].as_str().unwrap().trim_end_matches('\0'), "{at}.name");
    let f = &w["Fun_Data"];
    let m = &f["motor_module"];
    let motors = ["left_motor_value", "right_motor_value", "lt_motor_value", "rt_motor_value"].map(|k| n(&m[k]) / 25);
    assert_eq!(p.fun.motors.map(u64::from), motors, "{at}.motors");
    let x = &f["extend_module"];
    assert_eq!(
        [p.fun.shift_enabled, p.fun.shift_value, p.fun.dpad_diagonal_lock, p.fun.xinput_abxy_change, p.fun.switch_abxy_change,
            p.fun.report_rate_gears, p.fun.sensor_quick_mode, p.fun.stick_resolution].map(u64::from),
        ["shift_en", "shift_value", "dpad_diagonal_lock_en", "xinput_abxy_change", "switch_abxy_change", "report_rates_gears",
            "sensor_quick_mode", "stick_resolution"].map(|k| n(&x[k])),
        "{at}.extend"
    );
    if let Some(j) = x.get("ls_AntiJitter") {
        assert_eq!((u64::from(p.fun.ls_anti_jitter), u64::from(p.fun.rs_anti_jitter)), (n(j), n(&x["rs_AntiJitter"])), "{at}.jitter");
    }
    let keys = &w["KeyProfiles"];
    let order = ["Up", "Under", "Left", "Right", "L1", "R1", "L3", "R3", "CRO", "CIR", "SQU", "TRI", "PS", "Sel", "Sta", "Camera"];
    for (b, k) in p.buttons.iter().zip(order) {
        key(&keys[k], b.turbo, b.mapped, b.map, b.toggle_on_press, &format!("{at}.{k}"));
    }
    let funcs: Vec<&FunctionKey> = [&p.fl1, &p.fr1].into_iter().chain(&p.extra).collect();
    assert_eq!(funcs.len(), fn_keys.len(), "{at}: function key count");
    for (fk, name) in funcs.into_iter().zip(fn_keys) {
        let w = &keys[*name];
        let at = format!("{at}.{name}");
        key(w, fk.turbo, fk.mapped, fk.map, fk.toggle_on_press, &at);
        let mf = &w["macro_fun"];
        let st = &mf["marco_open_status"];
        assert_eq!((fk.macro_enabled as u64, fk.macro_press_to_trigger as u64, fk.macro_cycle as u64),
            (n(&st["marco_en"]), n(&st["marco_key_open"]), n(&st["marco_cycle_en"])), "{at}.flags");
        assert_eq!(u64::from(fk.cycle_ms), n(&mf["marco_cycle_time_h"]) * 256 + n(&mf["marco_cycle_time_l"]), "{at}.cycle");
        assert_eq!(fk.steps.len() as u64, n(&mf["step_num"]), "{at}.steps");
        for (i, s) in fk.steps.iter().enumerate() {
            let ws = &mf["steps"][i];
            let delay = ws.get("step_delay_time_h").map_or(0, |h| n(h) * 256 + n(&ws["step_delay_time_l"]));
            assert_eq!((u64::from(s.key), u64::from(s.hold_ms), u64::from(s.delay_ms)),
                (n(&ws["step_data"]), n(&ws["step_hold_time_h"]) * 256 + n(&ws["step_hold_time_l"]), delay), "{at}.step{i}");
        }
    }
    for (t, k) in [(&p.l2, "L2"), (&p.r2, "R2")] {
        let w = &keys[k];
        let at = format!("{at}.{k}");
        key(w, t.turbo, t.mapped, t.map, t.toggle_on_press, &at);
        dead(&t.dead, &w["dead_module"], &at);
        let q = &w["quick_trigger"];
        assert_eq!([t.quick_trigger_status, t.quick_trigger_start, t.quick_trigger_end].map(u64::from),
            ["quick_trigger_status", "quick_trigger_start_value", "quick_trigger_end_value"].map(|k| n(&q[k])), "{at}.quick");
        linear(&t.linear, &w["linear_module"], &at);
    }
    for (s, k) in [(&p.left_stick, "Left"), (&p.right_stick, "Right")] {
        let w = &w["Sticks"][k];
        let at = format!("{at}.stick{k}");
        assert_eq!((s.enabled as u64, s.square as u64), (n(&w["stick_en"]), n(&w["stick_square"])), "{at}.flags");
        dead(&s.dead, &w["dead_module"], &at);
        linear(&s.linear, &w["linear_module"], &at);
        map(&s.map, &w["map_module"], &at);
    }
    for (m, k) in [(&p.aim, "Aim"), (&p.tilt, "Tilt")] {
        let w = &w["Motion"][k];
        let at = format!("{at}.{k}");
        assert_eq!(
            [m.sensor_profile_status, m.sensor_quick_key_value, m.quick_key_value2, m.quick_key_value3, m.active_axis].map(u64::from),
            ["sensor_profile_status", "sensor_quick_key_value", "sensor_quick_key_value2", "sensor_quick_key_value3", "active_axis"].map(|k| n(&w[k])),
            "{at}.head"
        );
        dead(&m.dead, &w["dead_module"], &at);
        linear(&m.linear, &w["linear_module"], &at);
        map(&m.map, &w["map_module"], &at);
        assert_eq!(u64::from(m.stick_sensitivity), n(&w["map_module"]["stickSensitivity"]), "{at}.sensitivity");
    }
    for (t, k) in p.touch.iter().zip(["Ext8", "Ext9"]) {
        let w = &keys[k];
        let at = format!("{at}.{k}");
        key(w, t.turbo, t.mapped, t.map, t.toggle_on_press, &at);
        assert_eq!((u64::from(t.trigger_mode), u64::from(t.sensitivity)), (n(&w["triggerMode"]), n(&w["sensitivity"])), "{at}.touch");
    }
}

#[test]
fn v2_profile_vectors_decode_like_the_vendor_and_round_trip() {
    let mut cases = 0;
    for (id, json) in VECTORS {
        let model = by_id(id).unwrap();
        let Family::V2(spec) = model.family else { panic!("{id} is not V2") };
        let v: Value = serde_json::from_str(json).unwrap();
        for c in v["profile"].as_array().unwrap() {
            let at = format!("{id}/{}", c["name"].as_str().unwrap());
            let bytes = hex(c["hex"].as_str().unwrap());
            let p = profile::decode_for(model, &bytes).unwrap_or_else(|e| panic!("{at}: {e}"));
            compare(&p, &c["raw_decoded"], spec.fn_keys, &at);
            same_bytes(&profile::encode_for(model, &p).unwrap(), &bytes, &format!("{at}: re-encode"));
            // Encoding from the model alone (no stored bytes) must give the same bytes too.
            let fresh = Profile { raw: Vec::new(), ..p.clone() };
            let again = profile::encode_for(model, &fresh).unwrap();
            assert_eq!(profile::decode_for(model, &again).unwrap(), Profile { raw: again.clone(), ..p }, "{at}: model round trip");
            cases += 1;
        }
    }
    assert_eq!(cases, 8 + 7 + 7);
}

#[test]
fn v2_light_vectors_decode_like_the_vendor_and_round_trip() {
    for (id, json) in VECTORS {
        let model = by_id(id).unwrap();
        let Family::V2(spec) = model.family else { unreachable!() };
        let v: Value = serde_json::from_str(json).unwrap();
        for c in v["light"].as_array().unwrap() {
            let at = format!("{id}/light/{}", c["name"].as_str().unwrap());
            let bytes = hex(c["hex"].as_str().unwrap());
            let l = effect_light::decode(model, &bytes).unwrap_or_else(|e| panic!("{at}: {e}"));
            let w = &c["raw_decoded"]["Light"];
            for (i, a) in l.animations.iter().enumerate() {
                let wa = &w[format!("Animation{}", i + 1)];
                assert_eq!(u64::from(a.mode), n(&wa["light"]["mode"]), "{at}.mode{i}");
                // T3CE names the two hue settings `*HueOffset`; position is what matters.
                let want: Vec<u64> = spec.light_settings.iter().map(|k| {
                    let w = &wa["light"];
                    n(w.get(*k).unwrap_or_else(|| &w[format!("{k}Offset")]))
                }).collect();
                assert_eq!(a.settings.iter().map(|&s| u64::from(s)).collect::<Vec<_>>(), want, "{at}.settings{i}");
                for (z, name) in a.zones.iter().zip(spec.light_zones) {
                    let wz = &wa["color"][*name];
                    assert_eq!((u64::from(z.h), u64::from(z.s), u64::from(z.v)), (n(&wz["h"]), n(&wz["s"]), n(&wz["v"])), "{at}.{name}");
                }
            }
            same_bytes(&effect_light::encode(model, &l).unwrap(), &bytes, &format!("{at}: re-encode"));
        }
    }
}

#[test]
fn v2_section_table_covers_the_vendor_layout() {
    for (id, json) in LAYOUTS {
        let model = by_id(id).unwrap();
        let v: Value = serde_json::from_str(json).unwrap();
        let len = profile::len_for(model).unwrap();
        assert_eq!(len as u64, n(&v["length"]), "{id}");
        // Every byte the vendor decoder reads (except reserved ones, kept in `raw`) must reach the
        // decoded model: toggling it changes the decode for some baseline (16 or 32 macro steps).
        let decode = |b: &[u8]| Profile { raw: Vec::new(), ..profile::decode_for(model, b).unwrap() };
        for f in v["fields"].as_array().unwrap() {
            let path = f["path"].as_str().unwrap();
            if path.contains("reserved") {
                continue;
            }
            let off = n(&f["offset"]) as usize;
            // 0x11/0x21 set every enable bit, so gated values (turbo speed) show too.
            let seen = [0x00u8, 0x10, 0x20, 0x11, 0x21].into_iter().any(|fill| {
                let base = vec![fill; len];
                [0x01u8, 0x40].into_iter().any(|flip| {
                    let mut b = base.clone();
                    b[off] ^= flip;
                    decode(&b) != decode(&base)
                })
            });
            assert!(seen, "{id}: {path} at {off} is not modelled");
        }
    }
}
