//! Replays `docs/protocol/sapjoy-gs3-vectors.json`: frames produced by running the vendor SAPJoy
//! T4N2 DLL (T3XS path) under emulation. Every request OpenSir builds must match byte for byte.
use opensir_core::gs3::{self, Cmd};
use serde_json::Value;

const VECTORS: &str = include_str!("../../../docs/protocol/sapjoy-gs3-vectors.json");

fn hex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

/// OpenSir's request for one vendor call: the struct bytes are decoded into our typed struct and
/// encoded again, so the test covers both the field layout and the framing.
fn ours(export: &str, args: &[u8], s: &[u8]) -> Option<Vec<u8>> {
    Some(match export {
        "SetGamepadStateReport" => gs3::set_gamepad_state_report(args[0]),
        "SetABXYRotationTest" => gs3::frame(Cmd::SetAbxyRotationTest, &args[..1]),
        "SetReportRate" => return None, // u16 argument: checked separately below
        "RecordMacro" => gs3::frame(Cmd::RecordMacro, &args[..2]),
        "SaveConfigPage" => gs3::frame(Cmd::SaveConfigPage, &[]),
        "SetKeyMappingHSV" => gs3::frame(Cmd::SetKeyMappingHsv, &gs3::KeyMapping::decode(s).encode()),
        "SetStickCurveHSV" => gs3::frame(Cmd::SetStickCurveHsv, &gs3::StickCurve::decode(s).encode()),
        "SetMotionSenseCurveHSV" => gs3::frame(Cmd::SetMotionSenseCurveHsv, &gs3::MotionCurve::decode(s).encode()),
        "SetTriggerParam" => gs3::frame(Cmd::SetTriggerParam, &gs3::TriggerParam::decode(s).encode()),
        "SetLightColorHSV" => gs3::frame(Cmd::SetLightColorHsv, &gs3::LightColor::decode(s).encode()),
        "SetMacroStep" => gs3::frame(Cmd::SetMacroStep, &gs3::MacroStep::decode(s).encode()),
        "SetStickMapping" => gs3::set_bytes(Cmd::SetStickMapping, s),
        "SetVibrateLevel" => gs3::set_bytes(Cmd::SetVibrateLevel, s),
        "SetLightParam" => gs3::set_bytes(Cmd::SetLightParam, s),
        "SetDPadParam" => gs3::set_bytes(Cmd::SetDPadParam, s),
        "SetVibrateTest" => gs3::set_bytes(Cmd::SetVibrateTest, s),
        "SetMotionSenseMapping" => gs3::set_bytes(Cmd::SetMotionSenseMapping, s),
        // The vendor ffi struct has 3 bytes; the firmware struct adds 6 reserved bytes.
        "SetGamepadParam" => gs3::set_bytes(Cmd::SetGamepadParam, &[s, &[0; 6]].concat()),
        // Reads: only the selector bytes the DLL forwards.
        "GetVersion" => gs3::frame(Cmd::GetVersion, &[]),
        "GetDeviceInfo" => gs3::frame(Cmd::GetDeviceInfo, &[]),
        "GetMacAddress" => gs3::frame(Cmd::GetMacAddress, &[]),
        "GetDeviceType" => gs3::frame(Cmd::GetDeviceType, &[]),
        "GetKeyMappingHSV" => gs3::frame(Cmd::GetKeyMappingHsv, &args[1..2]),
        "GetStickCurveHSV" => gs3::frame(Cmd::GetStickCurveHsv, &args[..1]),
        "GetStickMapping" => gs3::frame(Cmd::GetStickMapping, &args[..1]),
        "GetTriggerParam" => gs3::frame(Cmd::GetTriggerParam, &args[..1]),
        "GetVibrateLevel" => gs3::frame(Cmd::GetVibrateLevel, &[]),
        // The vendor DLL reads the light parameters with command 15 (GetLightColorHSV id), not 13.
        "GetLightParam" => gs3::frame(Cmd::GetLightColorHsv, &[]),
        "GetDPadParam" => gs3::frame(Cmd::GetDPadParam, &[]),
        "GetMotionSenseCurveHSV" => gs3::frame(Cmd::GetMotionSenseCurveHsv, &[]),
        "GetMotionSenseMapping" => gs3::frame(Cmd::GetMotionSenseMapping, &[]),
        "GetMacroStep" => gs3::frame(Cmd::GetMacroStep, &args[..2]),
        "GetGamepadParam" => gs3::frame(Cmd::GetGamepadParam, &[]),
        "GetReportRate" => gs3::frame(Cmd::GetReportRate, &[]),
        // The DLL sends a whole (uninitialised) colour struct as the read selector; not modelled.
        "GetLightColorHSV" => return None,
        other => panic!("vector for an unexpected export {other}"),
    })
}

#[test]
fn requests_match_the_vendor_dll() {
    let v: Value = serde_json::from_str(VECTORS).unwrap();
    let mut checked = 0;
    for c in v["vectors"].as_array().unwrap() {
        let export = c["export"].as_str().unwrap();
        let frames = c["frames"].as_array().unwrap_or_else(|| panic!("{export}: {}", c["error"]));
        assert_eq!(frames.len(), 1, "{export}: one frame per call");
        let want = hex(frames[0].as_str().unwrap());
        let args: Vec<u8> = c["args"].as_array().map_or(Vec::new(), |a| a.iter().map(|x| x.as_u64().unwrap() as u8).collect());
        let s = c["struct_hex"].as_str().map(hex).unwrap_or_default();
        if export == "SetReportRate" {
            let hz = c["args"][0].as_u64().unwrap() as u16;
            assert_eq!(gs3::set_report_rate(hz), want, "SetReportRate {hz}");
            checked += 1;
            continue;
        }
        if let Some(got) = ours(export, &args, &s) {
            assert_eq!(got, want, "{export} {}", c["case"]);
            checked += 1;
        }
    }
    assert_eq!(checked, 10 + 2 * 13 + 16);
}

#[test]
fn typed_structs_round_trip() {
    let s: Vec<u8> = (0..22u8).map(|i| i * 7 + 3).collect();
    assert_eq!(gs3::KeyMapping::decode(&s).encode(), s[..17]);
    assert_eq!(gs3::StickCurve::decode(&s).encode(), s[..22]);
    assert_eq!(gs3::MotionCurve::decode(&s).encode(), s[..20]);
    assert_eq!(gs3::TriggerParam::decode(&s).encode(), s[..16]);
    assert_eq!(gs3::LightColor::decode(&s).encode(), s[..17]);
    assert_eq!(gs3::MacroStep::decode(&s).encode(), s[..16]);
}

#[test]
fn reply_payload_checks_header_and_command() {
    let r = [0xA5, 8, 0xC1, 3, 0, 1, 2, 3];
    assert_eq!(gs3::reply_payload(Cmd::GetVersion, &r), Some(&[1u8, 2, 3][..]));
    assert_eq!(gs3::reply_payload(Cmd::GetDeviceInfo, &r), None);
}

/// `docs/protocol/sapjoy-t3pro-vectors.json`: the T3Pro/NovaPro library's frames for the commands
/// OpenSir models.
#[test]
fn t3pro_frames_match_the_vendor_dll() {
    use opensir_core::t3pro::{self, Cmd as T};
    let v: Value = serde_json::from_str(include_str!("../../../docs/protocol/sapjoy-t3pro-vectors.json")).unwrap();
    let mut checked = 0;
    for c in v["vectors"].as_array().unwrap() {
        let cmd = match c["export"].as_str().unwrap() {
            "SetStickMapping" => T::SetStickMapping,
            "SetTriggerParam" => T::SetTriggerParam,
            "SetMotionSenseMapping" => T::SetMotionSenseMapping,
            "SetVibrateLevel" => T::SetVibrateLevel,
            "SetLightParam" => T::SetLightParam,
            _ => continue,
        };
        let want = hex(c["frames"][0].as_str().unwrap());
        assert_eq!(t3pro::frame(cmd, &hex(c["struct_hex"].as_str().unwrap()))[..], want[..], "{cmd:?} {}", c["case"]);
        checked += 1;
    }
    assert_eq!(checked, 10);
}
