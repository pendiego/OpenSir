//! GameSir models from the vendor device table (`docs/protocol/other-models.md`).
//! The Cyclone 2 and the HID family (`Family::V2`: T3CE, G7ProCE, K2_8K) have a protocol
//! implementation. The others are recognised so the app can name them when they are plugged in.

/// How the vendor app talks to the model (other-models.md, "Overview").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// Vendor-defined HID collection (usage page 0xFFF0), like the Cyclone 2.
    Hid,
    /// Through the vendor's SAPJoy DLL; no HID reports are built by the app.
    SapJoy,
}

/// Which codec family OpenSir uses for the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// 680-byte profile, 635-byte key-frame light (`cyclone2-*.md`).
    C2,
    /// Wider sections, 16-bit dead zones, effect light (`docs/protocol/layouts/`).
    V2(&'static V2Spec),
    /// Recognised only; nothing is sent to it.
    Unsupported,
}

/// Per-model facts of the V2 family. Names follow `docs/protocol/layouts/*.json`.
#[derive(Debug, PartialEq, Eq)]
pub struct V2Spec {
    pub profile_len: usize,
    pub light_len: usize,
    /// Function keys from offset 176 (169 bytes each); the first two are FL1, FR1.
    pub fn_keys: &'static [&'static str],
    /// Fun extend block carries left/right stick anti-jitter after stick resolution.
    pub anti_jitter: bool,
    /// Two 10-byte touch keys (Ext8, Ext9) after the motion packets.
    pub touch_keys: bool,
    /// Per-animation u8 settings after the mode byte.
    pub light_settings: &'static [&'static str],
    /// HSV zones per animation (h u16 big-endian, s, v).
    pub light_zones: &'static [&'static str],
    /// Tail bytes after the RaiseWakeUp byte, in order.
    pub light_tail: &'static [&'static str],
}

#[derive(Debug)]
pub struct Model {
    /// Vendor `ProductType`; the Cyclone 2 keeps OpenSir's historic "C2".
    pub id: &'static str,
    pub name: &'static str,
    pub transport: Transport,
    pub family: Family,
    pub wired: &'static [u16],
    pub wireless: &'static [u16],
}

const EFFECT_SETTINGS: &[&str] = &[
    "singleColorBrightness", "breathingBrightness", "phantomBrightness", "rainbowBrightness", "radarBrightness",
    "breathingSpeed", "phantomSpeed", "rainbowSpeed", "radarSpeed", "phantomHue", "rainbowHue",
];
const K2_SETTINGS: &[&str] = &[
    "homeRgbEffectIndex", "profileNumRgbEffectIndex", "homeRgbPhantomSpeed", "profileNumRgbPhantomSpeed",
    "singleColorBrightness", "breathingBrightness", "phantomBrightness", "rainbowBrightness", "radarBrightness",
    "breathingSpeed", "phantomSpeed", "rainbowSpeed", "radarSpeed", "phantomHue", "rainbowHue",
];

pub static T3CE: V2Spec = V2Spec {
    profile_len: 1935,
    light_len: 102,
    fn_keys: &["FL1", "FR1", "Ext1", "Ext2", "Ext3", "Ext4", "Ext5", "Ext6", "Ext7"],
    anti_jitter: false,
    touch_keys: true,
    light_settings: EFFECT_SETTINGS,
    light_zones: &["panel", "home"],
    light_tail: &["standbyTime"],
};
pub static G7PROCE: V2Spec = V2Spec {
    profile_len: 1070,
    light_len: 165,
    fn_keys: &["FL1", "FR1", "FL2", "FR2"],
    anti_jitter: true,
    touch_keys: false,
    light_settings: EFFECT_SETTINGS,
    light_zones: &["lowerLeft", "upperLeft", "upperRight", "lowerRight", "home"],
    light_tail: &["standbyTime", "dockLedMode", "dockLedBrightness", "reportRate"],
};
pub static K2_8K: V2Spec = V2Spec {
    profile_len: 1408,
    light_len: 229,
    fn_keys: &["FL1", "FR1", "FL2", "FR2", "FL3", "FR3"],
    anti_jitter: true,
    touch_keys: false,
    light_settings: K2_SETTINGS,
    light_zones: &["lowerLeft", "lowerRight", "upperLeft", "upperRight", "home", "profileNum", "homeButton"],
    light_tail: &["standbyTime", "dockLedMode", "dockLedBrightness", "reportRate", "deviceId0", "deviceId1", "deviceId2", "deviceId3"],
};

pub const MODELS: &[Model] = &[
    Model { id: "C2", name: "GameSir Cyclone 2", transport: Transport::Hid, family: Family::C2, wired: &[0x101D, 0x1053], wireless: &[0x102A, 0x100B] },
    Model { id: "T3Lite", name: "GameSir Tegenaria Lite", transport: Transport::Hid, family: Family::Unsupported, wired: &[0x1093], wireless: &[] },
    Model { id: "K2_8K", name: "GameSir K2 8K", transport: Transport::Hid, family: Family::V2(&K2_8K), wired: &[0x108E], wireless: &[0x109F] },
    Model { id: "T3CE", name: "GameSir Tarantula CE", transport: Transport::Hid, family: Family::V2(&T3CE), wired: &[0x103D, 0x090D], wireless: &[] },
    Model {
        id: "G7ProCE",
        name: "GameSir G7 Pro 8K",
        transport: Transport::Hid,
        family: Family::V2(&G7PROCE),
        wired: &[0x10B7, 0x10CD, 0x10C5, 0x10FD, 0x10C7, 0x10C9, 0x10CB, 0x1032, 0x1034, 0x10FB],
        wireless: &[0x10B9, 0x10CE, 0x10C6, 0x10FE, 0x10C8, 0x10CA, 0x10CC, 0x1033, 0x1035, 0x10FC],
    },
    Model { id: "T3Pro", name: "GameSir Tarantula Pro", transport: Transport::SapJoy, family: Family::Unsupported, wired: &[0x1050], wireless: &[] },
    Model { id: "T4NPro", name: "GameSir Nova Pro", transport: Transport::SapJoy, family: Family::Unsupported, wired: &[0x100E], wireless: &[] },
    // 0x103E is the wireless PID of both the Tarantula Pro and the Nova Pro; the PID alone cannot tell them apart.
    Model { id: "T3Pro_T4NPro", name: "GameSir Tarantula Pro / Nova Pro", transport: Transport::SapJoy, family: Family::Unsupported, wired: &[], wireless: &[0x103E] },
    Model { id: "T4N2", name: "GameSir Nova 2 Lite", transport: Transport::SapJoy, family: Family::Unsupported, wired: &[0x100F], wireless: &[0x1098] },
    Model { id: "N2S", name: "GameSir Nova Vision", transport: Transport::SapJoy, family: Family::Unsupported, wired: &[0x1141], wireless: &[0x10BD] },
    Model { id: "T3XS", name: "GameSir T3XS", transport: Transport::SapJoy, family: Family::Unsupported, wired: &[0x105A], wireless: &[0x10BE] },
];

/// The model a GameSir PID belongs to, and whether that PID is its wireless one.
pub fn by_pid(pid: u16) -> Option<(&'static Model, bool)> {
    MODELS.iter().find_map(|m| {
        if m.wired.contains(&pid) {
            Some((m, false))
        } else if m.wireless.contains(&pid) {
            Some((m, true))
        } else {
            None
        }
    })
}

pub fn by_id(id: &str) -> Option<&'static Model> {
    MODELS.iter().find(|m| m.id == id)
}

/// PIDs whose vendor `features` block disables keyboard/mouse mapping, map-to-more-keys and
/// turbo, and hides the motion page (other-models.md: T3CE 0x090D, G7ProCE 0x10CD/0x10CE).
pub fn restricted(pid: u16) -> bool {
    matches!(pid, 0x090D | 0x10CD | 0x10CE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pids_are_unique_and_match_the_survey() {
        let mut all: Vec<u16> = MODELS.iter().flat_map(|m| m.wired.iter().chain(m.wireless)).copied().collect();
        let n = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), n, "a PID is listed twice");
        // Decimal PIDs from the G7 Pro 8K table in other-models.md.
        assert_eq!(by_pid(4279).map(|(m, w)| (m.id, w)), Some(("G7ProCE", false)));
        assert_eq!(by_pid(4348).map(|(m, w)| (m.id, w)), Some(("G7ProCE", true)));
        assert_eq!(by_pid(0x102A).map(|(m, w)| (m.id, w)), Some(("C2", true)));
        assert!(by_pid(0x0575).is_none());
    }

    #[test]
    fn v2_sizes_add_up() {
        for spec in [&T3CE, &G7PROCE, &K2_8K] {
            let touch = if spec.touch_keys { 20 } else { 0 };
            assert_eq!(176 + 169 * spec.fn_keys.len() + 2 * 32 + 2 * 36 + 2 * 41 + touch, spec.profile_len);
            let anim = 1 + spec.light_settings.len() + 4 * spec.light_zones.len();
            assert_eq!(5 * anim + 1 + spec.light_tail.len(), spec.light_len);
        }
    }
}
