//! Outbound command builders and inbound frame classifier.
//! Frames are 64 bytes. Outbound frames start with report id 0x0F. Inbound config replies are
//! every report that is not the 0x12 gamepad-state report (the vendor log shows them on report
//! 0x10); their class byte is at index 1 (`docs/protocol/cyclone2-transport.md` §6, §8a).
//! Upgrade commands (`0f fc`, `0f 17 55 88`) are deliberately not built.

pub const FRAME_LEN: usize = 64;

/// `0f 01`: enter profile config mode.
pub fn enter_profile_config() -> Vec<u8> {
    frame(&[0x0f, 0x01])
}

/// `0f 02 00`: exit profile config mode.
pub fn exit_profile_config() -> Vec<u8> {
    frame(&[0x0f, 0x02, 0x00])
}

/// `0f 04 01 03 9d 07`: read device info block 1.
pub fn read_device_info_1() -> Vec<u8> {
    frame(&[0x0f, 0x04, 0x01, 0x03, 0x9d, 0x07])
}

/// `0f 0d 01 03`: set light status (1, 3).
pub fn set_light_status() -> Vec<u8> {
    frame(&[0x0f, 0x0d, 0x01, 0x03])
}

/// `0f f2 e`: heartbeat; `e` = 1 keeps the live input (button test) stream on, 0 turns it off.
pub fn heartbeat(test: bool) -> Vec<u8> {
    frame(&[0x0f, 0xf2, test as u8])
}

/// Read `len` bytes at `offset` of block `block` (profiles 1..=4, light 0x20). Offset is big-endian; chunks are 58 bytes.
pub fn read_chunk(block: u8, offset: u16, len: u8) -> Vec<u8> {
    let o = offset.to_be_bytes();
    frame(&[0x0f, 0x04, block, o[0], o[1], len])
}

/// Classify an inbound frame by its class byte at index 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inbound {
    /// The 0x12 gamepad-state report or an empty frame: not a config reply.
    Foreign,
    /// Class byte at index 1 (the caller decodes the payload).
    Class(u8),
}

pub fn classify(frame: &[u8]) -> Inbound {
    match frame {
        [crate::input::REPORT_ID, ..] | [] | [_] => Inbound::Foreign,
        [_, class, ..] => Inbound::Class(*class),
    }
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut f = vec![0u8; FRAME_LEN];
    f[..bytes.len()].copy_from_slice(bytes);
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_64_bytes_and_prefixed() {
        for f in [enter_profile_config(), exit_profile_config(), read_device_info_1(), set_light_status(), heartbeat(false)] {
            assert_eq!(f.len(), FRAME_LEN);
            assert_eq!(f[0], 0x0f);
        }
    }

    #[test]
    fn known_command_bytes() {
        assert_eq!(&enter_profile_config()[..2], &[0x0f, 0x01]);
        assert_eq!(&exit_profile_config()[..3], &[0x0f, 0x02, 0x00]);
        assert_eq!(&read_device_info_1()[..6], &[0x0f, 0x04, 0x01, 0x03, 0x9d, 0x07]);
        assert_eq!(&set_light_status()[..4], &[0x0f, 0x0d, 0x01, 0x03]);
    }

    #[test]
    fn classifies_inbound() {
        assert_eq!(classify(&[0x10, 0x06, 0]), Inbound::Class(0x06));
        assert_eq!(classify(&[0x12, 0x01, 0]), Inbound::Foreign);
    }
}
