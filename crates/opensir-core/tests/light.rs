use opensir_core::light::{decode, encode};
use serde_json::Value;

const VECTORS: &str = include_str!("../../../docs/protocol/cyclone2-light-shift-vectors.json");

#[test]
fn light_vectors_decode_and_roundtrip() {
    let fixture: Value = serde_json::from_str(VECTORS).unwrap();
    let vectors = fixture["vectors"].as_array().unwrap();
    assert_eq!(vectors.len(), 16);
    for v in vectors {
        let name = v["name"].as_str().unwrap();
        let hex = v["hex"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len()).step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect();
        let profile = decode(&bytes).unwrap();
        let actual = serde_json::to_value(&profile).unwrap();
        let want = &v["decoded_json"];
        assert_eq!(actual["configIndex"], want["configIndex"], "{name}");
        let tail = &want["tail"];
        assert_eq!(actual["audioReactive"], tail["audioReactiveMode"] == 1, "{name}");
        assert_eq!(actual["raiseWakeUp"], tail["raiseWakeUp"] == 1, "{name}");
        for key in ["userEffectIndex", "profileLed", "standbyTime"] {
            assert_eq!(actual[key], tail[key], "{name}: {key}");
        }
        assert_eq!(profile.animations.len(), 5, "{name}");
        for i in 0..5 {
            let a = &actual["animations"][i];
            let b = &want["animations"][i];
            for key in ["keyFrameCount", "effectCount", "brightness", "frames"] {
                assert_eq!(a[key], b[key], "{name}: {i}.{key}");
            }
            assert_eq!(a["speed"], 20 - b["speedRaw"].as_u64().unwrap(), "{name}");
        }
        assert_eq!(encode(&profile).unwrap(), bytes, "{name}");
    }
}

#[test]
fn rejects_bad_size_and_preserves_unknown_bits_on_edit() {
    assert!(decode(&[0; 634]).is_err());
    let mut bytes = [0; 635];
    bytes[621] = 0b1010;
    bytes[634] = 77;
    let mut p = decode(&bytes).unwrap();
    p.audio_reactive = true;
    let encoded = encode(&p).unwrap();
    assert_eq!((encoded[621], encoded[634]), (0b1011, 77));
    p.animations[0].key_frame_count = 9;
    assert!(encode(&p).is_err());
}
