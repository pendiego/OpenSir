//! Cyclone 2 lighting slot (32), as described in `docs/protocol/cyclone2-light.md`.

use crate::model::{LightAnimation, LightProfile};

pub const LIGHT_LEN: usize = 635;
const ANIMATIONS: usize = 5;
const FRAMES: usize = 8;
const ANIM_LEN: usize = 124;
const TAIL: usize = 1 + ANIMATIONS * ANIM_LEN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightError {
    BadLength(usize),
    BadAnimationCount(usize),
    BadFrameCount(usize),
    BadStandby(u8),
}

impl std::fmt::Display for LightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLength(n) => write!(f, "light block must be {LIGHT_LEN} bytes, got {n}"),
            Self::BadAnimationCount(n) => write!(f, "light block must have {ANIMATIONS} animations, got {n}"),
            Self::BadFrameCount(n) => write!(f, "light animation has an invalid frame count: {n}"),
            Self::BadStandby(n) => write!(f, "standby time must be 0, 1, 5, 10 or 20 minutes, got {n}"),
        }
    }
}

impl std::error::Error for LightError {}

fn config_index(value: u8) -> u8 {
    if value <= 3 { value } else { 0 }
}

pub fn decode(bytes: &[u8]) -> Result<LightProfile, LightError> {
    if bytes.len() != LIGHT_LEN {
        return Err(LightError::BadLength(bytes.len()));
    }
    let animations = bytes[1..TAIL].as_chunks::<ANIM_LEN>().0.iter().map(|a| LightAnimation {
        key_frame_count: a[0].min(FRAMES as u8),
        effect_count: a[1],
        speed: 20u8.saturating_sub(a[2]),
        brightness: a[3].min(100),
        frames: a[4..].as_chunks::<15>().0.iter().map(|frame| {
            std::array::from_fn(|i| [frame[i * 3], frame[i * 3 + 1], frame[i * 3 + 2]])
        }).collect(),
    }).collect();
    Ok(LightProfile {
        config_index: config_index(bytes[0]),
        animations,
        audio_reactive: bytes[TAIL] & 1 != 0,
        user_effect_index: bytes[TAIL + 1],
        profile_led: [bytes[TAIL + 2], bytes[TAIL + 3], bytes[TAIL + 4]],
        raise_wake_up: bytes[TAIL + 5] & 1 != 0,
        standby_time: bytes[TAIL + 6],
        raw: bytes.to_vec(),
    })
}

pub fn encode(profile: &LightProfile) -> Result<Vec<u8>, LightError> {
    if profile.animations.len() != ANIMATIONS {
        return Err(LightError::BadAnimationCount(profile.animations.len()));
    }
    let mut out = match profile.raw.len() {
        0 => vec![0; LIGHT_LEN],
        LIGHT_LEN => profile.raw.clone(),
        n => return Err(LightError::BadLength(n)),
    };
    out[0] = config_index(profile.config_index);
    for (a, bytes) in profile.animations.iter().zip(out[1..TAIL].as_chunks_mut::<ANIM_LEN>().0) {
        if a.frames.len() > FRAMES {
            return Err(LightError::BadFrameCount(a.frames.len()));
        }
        // The vendor never leaves an animation without a frame (a 0 already on the device is kept).
        let emptied = a.key_frame_count == 0 && bytes[0] != 0;
        if emptied || a.key_frame_count as usize > a.frames.len() {
            return Err(LightError::BadFrameCount(a.key_frame_count.into()));
        }
        bytes[..4].copy_from_slice(&[a.key_frame_count, a.effect_count, 20u8.saturating_sub(a.speed), a.brightness.min(100)]);
        for (frame, slot) in a.frames.iter().zip(bytes[4..].as_chunks_mut::<15>().0) {
            for (color, rgb) in frame.iter().zip(slot.as_chunks_mut::<3>().0) {
                rgb.copy_from_slice(color);
            }
        }
    }
    out[TAIL] = out[TAIL] & !1 | u8::from(profile.audio_reactive);
    out[TAIL + 1] = profile.user_effect_index;
    out[TAIL + 2..TAIL + 5].copy_from_slice(&profile.profile_led);
    out[TAIL + 5] = out[TAIL + 5] & !1 | u8::from(profile.raise_wake_up);
    // Vendor choices only (minutes); a value already on the device is kept as is.
    if ![0, 1, 5, 10, 20].contains(&profile.standby_time) && profile.standby_time != out[TAIL + 6] {
        return Err(LightError::BadStandby(profile.standby_time));
    }
    out[TAIL + 6] = profile.standby_time;
    Ok(out)
}
