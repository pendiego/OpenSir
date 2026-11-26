//! Page-level reset defaults, excluding lights, audio, and profile metadata.

use crate::{keycodes::key_code, model::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetError(String);

impl std::fmt::Display for ResetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown profile reset scope {:?}", self.0)
    }
}

impl std::error::Error for ResetError {}

/// Cyclone 2 units (dead zones 0..=100).
pub fn apply(p: &mut Profile, scope: &str) -> Result<(), ResetError> {
    apply_scaled(p, scope, 1)
}

/// `dead_scale` multiplies the default dead-zone bounds: 1 for Cyclone 2, 10 for the V2 family (0..=1000).
pub fn apply_scaled(p: &mut Profile, scope: &str, dead_scale: u16) -> Result<(), ResetError> {
    let groups = ["mapping", "sticks", "motion", "triggers", "motors"];
    let all = scope.is_empty() || scope.eq_ignore_ascii_case("all");
    let includes = |group: &str| all || scope.eq_ignore_ascii_case(group);
    if !groups.into_iter().any(includes) {
        return Err(ResetError(scope.into()));
    }
    if includes("mapping") {
        p.buttons = Default::default();
        p.fl1 = Default::default();
        p.fr1 = Default::default();
        p.extra.iter_mut().for_each(|k| *k = Default::default());
        p.touch.iter_mut().for_each(|k| *k = TouchKey { trigger_mode: k.trigger_mode, sensitivity: k.sensitivity, ..Default::default() });
        for trigger in [&mut p.l2, &mut p.r2] {
            trigger.turbo = 0;
            trigger.mapped = false;
            trigger.map = [0; 3];
            trigger.toggle_on_press = false;
        }
    }
    if includes("sticks") {
        for (stick, map_index) in [(&mut p.left_stick, 1), (&mut p.right_stick, 2)] {
            let (dead, linear) = dead_and_curve(10, 100, dead_scale);
            *stick = Stick { enabled: true, square: false, dead, linear, map: default_map(map_index) };
        }
    }
    if includes("motion") {
        for (motion, map_index) in [(&mut p.aim, 1), (&mut p.tilt, 2)] {
            let (dead, linear) = dead_and_curve(10, 100, dead_scale);
            *motion = Motion {
                sensor_profile_status: 0,
                sensor_quick_key_value: key_code("NC"),
                active_axis: 2, // Orientation 0 sets bit 1 on the wire.
                dead, linear, map: default_map(map_index),
                ..Default::default()
            };
        }
    }
    if includes("triggers") {
        for trigger in [&mut p.l2, &mut p.r2] {
            (trigger.dead, trigger.linear) = dead_and_curve(5, 95, dead_scale);
            trigger.quick_trigger_status = 0;
            trigger.quick_trigger_start = 1;
            trigger.quick_trigger_end = 100;
        }
    }
    if includes("motors") {
        p.fun.motors = [3; 4];
    }
    Ok(())
}

fn dead_and_curve(front: u8, back: u8, scale: u16) -> (DeadZone, Linear) {
    let dead = DeadZone {
        enabled: true,
        front: u16::from(front) * scale,
        back: u16::from(back) * scale,
        anti_front: 0,
        anti_back: 100 * scale,
    };
    let mut points = [(0, 0), (0, 0), (0, 0), (0, 0), (255, 255)];
    let width = u16::from(back - front);
    for (i, value) in [25u16, 50, 75].into_iter().enumerate() {
        let x = ((value - u16::from(front)) * 255 + width / 2) / width;
        let y = (value * 255 + 50) / 100;
        points[i + 1] = (x as u8, y as u8);
    }
    (dead, Linear { enabled: true, curve_type: 0, curvature: 100, points })
}

fn default_map(map_index: u8) -> MapModule {
    MapModule { enabled: true, axis_ratio: 50, mouse_dpi: 50, map_index, ..Default::default() }
}
