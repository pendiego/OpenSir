//! Session behaviour over a fake HID link: block reads, diff writes, ack resend, slot checks.
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use opensir_core::device::Session;
use opensir_core::light::LIGHT_LEN;
use opensir_core::profile::{self, PROFILE_LEN};
use opensir_core::model::DeviceSnapshot;
use opensir_core::transport::{Error, Frame, Link};
use serde_json::Value;

const PROFILES: &str = include_str!("../../../docs/protocol/cyclone2-profile-vectors.json");
const LIGHTS: &str = include_str!("../../../docs/protocol/cyclone2-light-shift-vectors.json");

#[derive(Default)]
struct World {
    store: HashMap<u8, Vec<u8>>,
    pending: VecDeque<Frame>,
    sent: Vec<Vec<u8>>,
    drop_acks: u32,
    /// Writes answered with AckWithBusy (`10 06 01`) and not stored.
    busy_acks: u32,
    /// Read requests left unanswered (readback timeout).
    drop_reads: u32,
    current: u8,
}

struct Fake(Arc<Mutex<World>>);

fn reply(class: u8, slot: u8, off: u16, data: &[u8]) -> Frame {
    let mut r = [0u8; 64];
    r[0] = 0x0A;
    r[1] = class;
    r[2] = slot;
    r[3..5].copy_from_slice(&off.to_be_bytes());
    r[5] = data.len() as u8;
    r[6..6 + data.len()].copy_from_slice(data);
    r
}

impl Link for Fake {
    fn send(&mut self, f: &[u8]) -> Result<(), Error> {
        let mut g = self.0.lock().unwrap();
        let w = &mut *g;
        w.sent.push(f.to_vec());
        if f[0] != 0x0F {
            return Ok(());
        }
        let slot = f[2];
        let off = u16::from_be_bytes([f[3], f[4]]) as usize;
        let len = f.get(5).copied().unwrap_or(0) as usize;
        match f[1] {
            0x04 if w.drop_reads > 0 => w.drop_reads -= 1,
            0x04 => {
                let data = w.store[&slot][off..off + len].to_vec();
                w.pending.push_back(reply(5, slot, off as u16, &data));
            }
            0x03 => {
                if w.busy_acks > 0 {
                    w.busy_acks -= 1;
                    w.pending.push_back(reply(6, 1, 0, &[]));
                    return Ok(());
                }
                w.store.get_mut(&slot).unwrap()[off..off + len].copy_from_slice(&f[6..6 + len]);
                if w.drop_acks > 0 {
                    w.drop_acks -= 1;
                } else {
                    w.pending.push_back(reply(6, 0, 0, &[]));
                }
            }
            0x09 => {
                let mut r = [0u8; 64];
                r[0] = 0x0A;
                r[1] = 10;
                // UTF-16LE "03400121" from byte 2.
                for (i, c) in b"03400121".iter().enumerate() {
                    r[2 + 2 * i] = *c;
                }
                w.pending.push_back(r);
            }
            0x0B => {
                let mut r = [0u8; 64];
                r[0] = 0x0A;
                r[1] = 12;
                r[2] = w.current;
                w.pending.push_back(r);
            }
            _ => {}
        }
        Ok(())
    }

    fn recv(&mut self, buf: &mut Frame, _timeout_ms: i32) -> Result<usize, Error> {
        match self.0.lock().unwrap().pending.pop_front() {
            Some(r) => {
                *buf = r;
                Ok(64)
            }
            None => Ok(0),
        }
    }
}

fn parse_hex(s: &str) -> Vec<u8> {
    let c: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    (0..c.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&c[i..i + 2], 16).unwrap())
        .collect()
}

/// A profile block that round-trips exactly, and a light block.
fn fixtures() -> (Vec<u8>, Vec<u8>) {
    let profiles: Value = serde_json::from_str(PROFILES).unwrap();
    let profile = profiles
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["length"].as_u64() == Some(PROFILE_LEN as u64) && e["roundtrip"].as_bool() == Some(true))
        .map(|e| parse_hex(e["hex"].as_str().unwrap()))
        .expect("roundtrip profile vector");
    let lights: Value = serde_json::from_str(LIGHTS).unwrap();
    let light = lights["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| parse_hex(e["hex"].as_str().unwrap()))
        .find(|b| b.len() == LIGHT_LEN)
        .expect("light vector");
    (profile, light)
}

fn setup() -> (Session, DeviceSnapshot, Arc<Mutex<World>>) {
    let (profile_block, light_block) = fixtures();
    let mut world = World { current: 3, ..Default::default() };
    for slot in 1..=5 {
        world.store.insert(slot, profile_block.clone());
    }
    world.store.insert(32, light_block);
    let world = Arc::new(Mutex::new(world));
    let (s, snap) = Session::attach("test", Box::new(Fake(world.clone()))).unwrap();
    (s, snap, world)
}

fn writes(sent: &[Vec<u8>]) -> Vec<&Vec<u8>> {
    sent.iter().filter(|f| f[0] == 0x0F && f[1] == 0x03).collect()
}

#[test]
fn attach_reads_all_blocks() {
    let (_s, snap, _w) = setup();
    assert_eq!(snap.profiles.len(), 4);
    assert_eq!(snap.firmware, "3.4.0");
    assert_eq!(snap.dongle_firmware.as_deref(), Some("1.2.1"));
    assert_eq!(snap.current_profile, 3);
    assert!(snap.shift.is_some() && snap.light.is_some());
}

#[test]
fn changed_save_sends_only_changed_chunks() {
    let (mut s, snap, w) = setup();
    let mut p = snap.profiles[0].clone();
    p.name = "Teste".into();
    let expect = profile::encode(&p).unwrap();
    w.lock().unwrap().sent.clear();
    s.save_profile(1, &p).unwrap();
    let g = w.lock().unwrap();
    let ws = writes(&g.sent);
    assert!(!ws.is_empty() && ws.len() < PROFILE_LEN.div_ceil(58));
    assert!(ws.iter().all(|f| f[5] as usize <= 58));
    assert_eq!(g.store[&1], expect);
}

#[test]
fn unchanged_save_sends_nothing() {
    let (mut s, snap, w) = setup();
    w.lock().unwrap().sent.clear();
    s.save_profile(2, &snap.profiles[1]).unwrap();
    assert!(writes(&w.lock().unwrap().sent).is_empty());
}

/// Real device blobs keep non-canonical bytes in disabled sub-fields (profile.md §12). A one-field
/// edit must write only that field, and a no-op save must write nothing.
#[test]
fn noncanonical_profile_edit_writes_only_the_edited_byte() {
    let profiles: Value = serde_json::from_str(PROFILES).unwrap();
    let noncanonical = profiles
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["source"] == "vendor-log" && e["roundtrip"].as_bool() == Some(false))
        .map(|e| parse_hex(e["hex"].as_str().unwrap()))
        .expect("non-canonical vendor-log profile");
    let (profile_block, light_block) = fixtures();
    let mut world = World { current: 1, ..Default::default() };
    for slot in 1..=5 {
        world.store.insert(slot, profile_block.clone());
    }
    world.store.insert(4, noncanonical.clone());
    world.store.insert(32, light_block);
    let w = Arc::new(Mutex::new(world));
    let (mut s, snap) = Session::attach("test", Box::new(Fake(w.clone()))).unwrap();

    w.lock().unwrap().sent.clear();
    s.save_profile(4, &snap.profiles[3]).unwrap();
    assert!(writes(&w.lock().unwrap().sent).is_empty(), "no-op save wrote bytes");

    let mut p = snap.profiles[3].clone();
    p.fun.motors[0] = if p.fun.motors[0] == 1 { 2 } else { 1 };
    s.save_profile(4, &p).unwrap();
    let g = w.lock().unwrap();
    let changed: Vec<usize> = (0..PROFILE_LEN).filter(|&i| g.store[&4][i] != noncanonical[i]).collect();
    assert_eq!(changed, vec![32]);
}

#[test]
fn unchanged_light_sends_nothing() {
    let (mut s, snap, w) = setup();
    w.lock().unwrap().sent.clear();
    s.save_light(snap.light.as_ref().unwrap()).unwrap();
    assert!(writes(&w.lock().unwrap().sent).is_empty());
}

#[test]
fn dropped_ack_resends_chunk() {
    let (mut s, snap, w) = setup();
    w.lock().unwrap().drop_acks = 1;
    let mut p = snap.profiles[0].clone();
    p.name = "Teste".into();
    s.save_profile(1, &p).unwrap();
    let g = w.lock().unwrap();
    assert_eq!(g.store[&1], profile::encode(&p).unwrap());
    assert!(writes(&g.sent).len() >= 2);
}

/// A save that runs out of acks must not drop the stored block: the next save would otherwise
/// rewrite every byte from an all-zero baseline and wipe unmodelled bytes.
#[test]
fn failed_save_keeps_stored_block() {
    let (mut s, snap, w) = setup();
    w.lock().unwrap().drop_acks = u32::MAX;
    let mut p = snap.profiles[0].clone();
    p.name = "Falha".into();
    assert!(matches!(s.save_profile(1, &p), Err(Error::NoReply)));
    {
        let mut g = w.lock().unwrap();
        g.drop_acks = 0;
        g.sent.clear();
    }
    p.name = "Ok".into();
    s.save_profile(1, &p).unwrap();
    let g = w.lock().unwrap();
    assert!(writes(&g.sent).len() < PROFILE_LEN.div_ceil(58), "rewrote the whole block");
    assert_eq!(g.store[&1], profile::encode(&p).unwrap());
}

/// The 1 s heartbeat is `[0f f2 e]` with `e` = test mode; sending 0 switched the button test off.
#[test]
fn heartbeat_repeats_test_mode_flag() {
    let (mut s, _snap, w) = setup();
    let last = |w: &Arc<Mutex<World>>| w.lock().unwrap().sent.last().unwrap()[..3].to_vec();
    s.set_test_mode(true).unwrap();
    s.heartbeat().unwrap();
    assert_eq!(last(&w), [0x0f, 0xf2, 1]);
    s.set_test_mode(false).unwrap();
    s.heartbeat().unwrap();
    assert_eq!(last(&w), [0x0f, 0xf2, 0]);
}

#[test]
fn rejects_slots_outside_1_to_5() {
    let (mut s, snap, _w) = setup();
    assert!(matches!(s.save_profile(0, &snap.profiles[0]), Err(Error::BadSlot(0))));
    assert!(matches!(s.switch_profile(6), Err(Error::BadSlot(6))));
}

#[test]
fn reset_profile_applies_scope_and_saves() {
    let (mut s, _snap, w) = setup();
    let r = s.reset_profile(1, "mapping").unwrap();
    assert_eq!(w.lock().unwrap().store[&1], profile::encode(&r).unwrap());
    assert!(matches!(s.reset_profile(1, "nope"), Err(Error::Reset(_))));
}

/// A burst of input reports (1000 Hz stream) must reach the sink coalesced, end on the newest
/// state, and not block config replies queued behind it.
#[test]
fn input_flood_is_coalesced_and_config_still_answers() {
    use opensir_core::device::Event;
    let (mut s, snap, w) = setup();
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    s.set_event_sink(Arc::new(move |e| {
        if let Event::Input(g) = e {
            sink.lock().unwrap().push(g);
        }
    }));
    s.set_test_mode(true).unwrap();
    {
        let mut g = w.lock().unwrap();
        for i in 0..1000u32 {
            let mut r = [0u8; 64];
            r[0] = 0x12;
            r[1] = (i % 256) as u8; // left stick x moves every report
            r[8] = 200; // LT held
            g.pending.push_back(r);
        }
    }
    // A save issued during the flood still gets its acks.
    let mut p = snap.profiles[0].clone();
    p.name = "flood".into();
    s.save_profile(1, &p).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !w.lock().unwrap().pending.is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    std::thread::sleep(std::time::Duration::from_millis(60));
    let got = events.lock().unwrap();
    assert!(!got.is_empty(), "no input event reached the sink");
    assert!(got.len() < 200, "{} events for 1000 reports: not coalesced", got.len());
    let last = got.last().unwrap();
    assert_eq!(last.lt, 200);
    assert_eq!(last.lx, opensir_core::input::parse("test", &{
        let mut r = [0u8; 64];
        r[0] = 0x12;
        r[1] = (999 % 256) as u8;
        r
    }).unwrap().lx);
}

/// neutral → A → neutral inside one drain batch: the tap must still be shown, then cleared.
#[test]
fn tap_inside_one_batch_is_shown_then_released() {
    use opensir_core::device::Event;
    let (mut s, _snap, w) = setup();
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    s.set_event_sink(Arc::new(move |e| {
        if let Event::Input(g) = e {
            sink.lock().unwrap().push(g.buttons);
        }
    }));
    s.set_test_mode(true).unwrap();
    let report = |face: u8| {
        let mut r = [0u8; 64];
        r[0] = 0x12;
        r[1..5].copy_from_slice(&[128; 4]);
        r[5] = face | 0x08; // low nibble 8 = d-pad released
        r
    };
    {
        let mut g = w.lock().unwrap();
        g.pending.push_back(report(0x00));
        g.pending.push_back(report(0x20)); // CRO (A) pressed
        g.pending.push_back(report(0x00));
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let got = events.lock().unwrap().clone();
        let shown = got.iter().any(|b| b.iter().any(|x| x == "CRO"));
        let cleared = got.last().is_some_and(|b| b.is_empty());
        if shown && cleared {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "tap lost or never released: {got:?}");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn firmware_reply_parses_device_and_dongle() {
    let mut r = [0u8; 64];
    let text: [u16; 8] = [0x30, 0x33, 0x34, 0x30, 0x30, 0x31, 0x32, 0x31];
    r[0] = 0x0A;
    r[1] = 10;
    for (i, c) in text.iter().enumerate() {
        r[2 + 2 * i..4 + 2 * i].copy_from_slice(&c.to_le_bytes());
    }
    let (fw, dongle) = opensir_core::device::parse_firmware(&r);
    assert_eq!(fw, "3.4.0");
    assert_eq!(dongle.as_deref(), Some("1.2.1"));
}

#[test]
fn firmware_dongle_none_when_zero() {
    let mut r = [0u8; 64];
    r[0] = 0x0A;
    r[1] = 10;
    for (i, c) in b"03400000".iter().enumerate() {
        r[2 + 2 * i] = *c;
    }
    let (_, dongle) = opensir_core::device::parse_firmware(&r);
    assert_eq!(dongle, None);
}

/// V2 family: every model opens with its own block lengths, decodes its vendor vector, and a
/// one-field edit writes only the chunk holding that field.
#[test]
fn v2_models_open_and_write_diffs() {
    use opensir_core::models::{by_id, Family};
    let sets = [
        ("T3CE", include_str!("../../../docs/protocol/t3ce-raw-vectors.json")),
        ("G7ProCE", include_str!("../../../docs/protocol/g7proce-raw-vectors.json")),
        ("K2_8K", include_str!("../../../docs/protocol/k2_8k-raw-vectors.json")),
    ];
    for (id, json) in sets {
        let model = by_id(id).unwrap();
        let Family::V2(spec) = model.family else { unreachable!() };
        let v: Value = serde_json::from_str(json).unwrap();
        let block = |kind: &str, name: &str| {
            v[kind].as_array().unwrap().iter().find(|c| c["name"] == name).map(|c| parse_hex(c["hex"].as_str().unwrap())).unwrap()
        };
        let profile_block = block("profile", "function-keys-macros");
        let mut light_block = block("light", "modes");
        // Wake byte (after the five animations) with bits the model does not carry: they must survive a save.
        let wake = 5 * (1 + spec.light_settings.len() + 4 * spec.light_zones.len());
        light_block[wake] = 0xA4;
        assert_eq!((profile_block.len(), light_block.len()), (spec.profile_len, spec.light_len), "{id}");
        let mut world = World { current: 2, ..Default::default() };
        for slot in 1..=5 {
            world.store.insert(slot, profile_block.clone());
        }
        world.store.insert(32, light_block.clone());
        let w = Arc::new(Mutex::new(world));
        let (mut s, snap) = Session::attach_for(model, "test", Box::new(Fake(w.clone()))).unwrap();
        assert_eq!(snap.model, id);
        assert_eq!(snap.profiles.len(), 4, "{id}");
        assert_eq!(snap.profiles[0].extra.len(), spec.fn_keys.len() - 2, "{id}: extra keys");
        assert!(snap.light.is_none() && snap.effect_light.is_some(), "{id}: light kind");
        assert!(matches!(s.save_profile(1, &snap.profiles[0]), Err(Error::ReadOnly)), "{id}: untested model must be read-only");
        s.allow_untested_writes();

        w.lock().unwrap().sent.clear();
        s.save_profile(1, &snap.profiles[0]).unwrap();
        assert!(writes(&w.lock().unwrap().sent).is_empty(), "{id}: no-op save wrote bytes");

        // Right-stick front dead zone: a u16 big-endian field near the end of the block.
        let mut p = snap.profiles[0].clone();
        p.right_stick.dead.front = 777;
        s.save_profile(1, &p).unwrap();
        {
            let g = w.lock().unwrap();
            let changed: Vec<usize> = (0..spec.profile_len).filter(|&i| g.store[&1][i] != profile_block[i]).collect();
            assert!(!changed.is_empty() && changed.len() <= 2, "{id}: {changed:?}");
            let stored = profile::decode_for(model, &g.store[&1]).unwrap();
            assert_eq!(stored.right_stick.dead.front, 777, "{id}: stored value");
            // u16 big-endian on the wire: 777 = 0x0309.
            assert!(g.store[&1].windows(2).any(|b| b == [0x03, 0x09]), "{id}: big-endian bytes");
            assert!(writes(&g.sent).iter().all(|f| f[2] == 1 && f[5] as usize <= 58), "{id}: frame shape");
        }

        // Through JSON like the UI (IPC): `raw` is not serialized, so the session must supply it.
        let mut light: opensir_core::model::EffectLight =
            serde_json::from_str(&serde_json::to_string(snap.effect_light.as_ref().unwrap()).unwrap()).unwrap();
        light.animations[0].zones[0].h = 321;
        s.save_effect_light(&light).unwrap();
        let g = w.lock().unwrap();
        let changed: Vec<usize> = (0..spec.light_len).filter(|&i| g.store[&32][i] != light_block[i]).collect();
        let z = 1 + spec.light_settings.len();
        assert!(changed.iter().all(|&i| i == z || i == z + 1) && !changed.is_empty(), "{id}: light diff {changed:?}");
        assert_eq!(g.store[&32][wake], 0xA4, "{id}: wake byte kept");
    }
}

/// A busy reply (`10 06 01`) is not a write ack: the chunk is resent and really lands.
#[test]
fn busy_ack_is_not_success() {
    let (mut s, snap, w) = setup();
    let mut p = snap.profiles[0].clone();
    p.fun.motors[0] = if p.fun.motors[0] == 4 { 0 } else { 4 };
    w.lock().unwrap().busy_acks = 1;
    s.save_profile(1, &p).unwrap();
    let g = w.lock().unwrap();
    let stored = profile::decode(&g.store[&1]).unwrap();
    assert_eq!(stored.fun.motors[0], p.fun.motors[0]);
    assert!(writes(&g.sent).len() >= 2, "busy chunk must be resent");
}

/// IPC/import values the vendor never writes are refused before anything is sent.
#[test]
fn out_of_range_edit_is_refused() {
    let (mut s, snap, w) = setup();
    let bad: [fn(&mut opensir_core::model::Profile); 4] = [
        |p| p.fun.motors[0] = 9,
        |p| p.buttons[0].map[0] = 199,
        |p| p.left_stick.dead.front = 150,
        |p| p.fun.report_rate_gears = 7,
    ];
    for edit in bad {
        let mut p = snap.profiles[0].clone();
        edit(&mut p);
        w.lock().unwrap().sent.clear();
        assert!(matches!(s.save_profile(1, &p), Err(Error::Codec(_))));
        assert!(writes(&w.lock().unwrap().sent).is_empty());
    }
}

/// A readback that times out leaves only the bytes this save sent in doubt: the next save resends
/// those and never rewrites the rest of the block.
#[test]
fn readback_timeout_keeps_writes_diff_only() {
    let (mut s, snap, w) = setup();
    let mut p = snap.profiles[0].clone();
    p.fun.motors[0] = if p.fun.motors[0] == 4 { 0 } else { 4 };
    w.lock().unwrap().drop_reads = u32::MAX;
    assert!(s.save_profile(1, &p).is_err(), "readback must time out");
    let span = |f: &Vec<u8>| {
        let off = usize::from(u16::from_be_bytes([f[3], f[4]]));
        off..off + usize::from(f[5])
    };
    let first: Vec<_> = {
        let mut g = w.lock().unwrap();
        g.drop_reads = 0;
        let spans = writes(&g.sent).into_iter().map(span).collect();
        g.sent.clear();
        spans
    };
    assert!(!first.is_empty());
    s.save_profile(1, &p).unwrap();
    let g = w.lock().unwrap();
    let again = writes(&g.sent);
    assert!(!again.is_empty(), "doubtful bytes are resent");
    for f in again {
        let r = span(f);
        assert!(first.iter().any(|a| a.start <= r.start && r.end <= a.end), "rewrote untouched bytes {r:?}");
    }
}
