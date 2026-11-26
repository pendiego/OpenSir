//! OpenSir core: GameSir protocol (Cyclone 2 and the V2 HID family), profile model and codecs.
//! Clean-room, written from `docs/protocol/`. Nothing here sends to a device.

pub mod command;
pub mod device;
pub mod effect_light;
pub mod gs3;
pub mod input;
pub mod keycodes;
pub mod light;
pub mod model;
pub mod models;
pub mod profile;
pub mod reset;
pub mod t3pro;
pub mod transport;
