//! Config frames of the T3Pro / NovaPro SAPJoy library (`docs/protocol/sapjoy-t3pro.md`): HID output
//! report `07 len group item payload..`, len = payload + 3, zero-padded to the 64-byte report.
//! Vectors from running the vendor DLL: `docs/protocol/sapjoy-t3pro-vectors.json`.
//!
//! shortcut: only the commands whose payload is the argument struct verbatim are modelled
//! (stick mapping, trigger, motion mapping, vibrate level, light parameters); macro steps, curves,
//! key mapping, light colour and d-pad rewrite their struct and stay vectors-only until decoded.

pub const REPORT_ID: u8 = 0x07;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    SetStickMapping,
    SetTriggerParam,
    SetMotionSenseMapping,
    SetVibrateLevel,
    SetLightParam,
}

impl Cmd {
    /// (group, item) bytes of the request.
    pub fn id(self) -> (u8, u8) {
        match self {
            Self::SetStickMapping => (0x02, 0x03),
            Self::SetTriggerParam => (0x03, 0x01),
            Self::SetMotionSenseMapping => (0x04, 0x03),
            Self::SetVibrateLevel => (0x06, 0x01),
            Self::SetLightParam => (0x07, 0x01),
        }
    }
}

/// 64-byte output report for `cmd` with the struct bytes as payload (`gs3` struct layouts).
pub fn frame(cmd: Cmd, payload: &[u8]) -> [u8; 64] {
    let (group, item) = cmd.id();
    let mut f = [0u8; 64];
    f[..4].copy_from_slice(&[REPORT_ID, (payload.len() + 3) as u8, group, item]);
    f[4..4 + payload.len()].copy_from_slice(payload);
    f
}
