//! V2 family light slot (32): five animations of `mode`, the model's u8 settings and HSV zones
//! (h u16 big-endian, s, v), then a RaiseWakeUp byte (bit 0) and the model's tail bytes.
//! Layout per model: `models::V2Spec`, checked against `docs/protocol/layouts/*-light.json`.

use crate::light::LightError;
use crate::model::{EffectAnimation, EffectLight, Hsv};
use crate::models::{Family, Model, V2Spec};

const ANIMATIONS: usize = 5;

fn spec(model: &Model) -> Result<&'static V2Spec, LightError> {
    match model.family {
        Family::V2(s) => Ok(s),
        _ => Err(LightError::BadLength(0)),
    }
}

fn anim_len(s: &V2Spec) -> usize {
    1 + s.light_settings.len() + 4 * s.light_zones.len()
}

pub fn decode(model: &Model, b: &[u8]) -> Result<EffectLight, LightError> {
    let s = spec(model)?;
    if b.len() != s.light_len {
        return Err(LightError::BadLength(b.len()));
    }
    let n = anim_len(s);
    let tail = ANIMATIONS * n;
    let animations = b[..tail]
        .chunks_exact(n)
        .map(|a| {
            let z = 1 + s.light_settings.len();
            EffectAnimation {
                mode: a[0],
                settings: a[1..z].to_vec(),
                zones: a[z..].as_chunks::<4>().0.iter().map(|c| Hsv { h: u16::from_be_bytes([c[0], c[1]]), s: c[2], v: c[3] }).collect(),
            }
        })
        .collect();
    Ok(EffectLight { animations, raise_wake_up: b[tail] & 1 != 0, tail: b[tail + 1..].to_vec(), raw: b.to_vec() })
}

pub fn encode(model: &Model, l: &EffectLight) -> Result<Vec<u8>, LightError> {
    let s = spec(model)?;
    if l.animations.len() != ANIMATIONS {
        return Err(LightError::BadAnimationCount(l.animations.len()));
    }
    let mut out = match l.raw.len() {
        0 => vec![0; s.light_len],
        n if n == s.light_len => l.raw.clone(),
        n => return Err(LightError::BadLength(n)),
    };
    let n = anim_len(s);
    let tail = ANIMATIONS * n;
    for (a, bytes) in l.animations.iter().zip(out[..tail].chunks_exact_mut(n)) {
        let z = 1 + s.light_settings.len();
        bytes[0] = a.mode;
        // Missing settings or zones keep the stored bytes.
        for (v, slot) in a.settings.iter().zip(&mut bytes[1..z]) {
            *slot = *v;
        }
        for (hsv, c) in a.zones.iter().zip(bytes[z..].as_chunks_mut::<4>().0) {
            c[..2].copy_from_slice(&hsv.h.to_be_bytes());
            c[2] = hsv.s;
            c[3] = hsv.v;
        }
    }
    out[tail] = out[tail] & !1 | u8::from(l.raise_wake_up);
    for (v, slot) in l.tail.iter().zip(&mut out[tail + 1..]) {
        *slot = *v;
    }
    Ok(out)
}
