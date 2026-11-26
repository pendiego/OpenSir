//! Cyclone 2 discovery and the configuration session (`docs/protocol/cyclone2-transport.md`,
//! `cyclone2-sequencing.md`). Unsupported devices are listed but never opened.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::command::{self, FRAME_LEN};
use crate::input;
use crate::effect_light;
use crate::model::EffectLight;
use crate::light::{self, LIGHT_LEN};
use crate::model::{DeviceInfo, DeviceMode, DeviceSnapshot, GamepadState, LightProfile, Profile};
use crate::models::{self, Family, Model};
use crate::profile::{self, PROFILE_LEN};
use crate::reset;
use crate::transport::{ATTEMPTS, Error, Frame, Link, SharedLink, WAIT, Wire, lock, pad, share, with_api};

pub const VENDOR_ID: u16 = 0x3537;
pub const USAGE_PAGE: u16 = 0xFFF0;

/// Cyclone 2 seen in a non-config mode (vendor app logs it with `nativeSupport: false`). Never opened.
pub const C2_NON_CONFIG_PID: u16 = 0x0575;

/// Payload bytes per WriteProfile/ReadProfile chunk.
const CHUNK: usize = 58;
/// Chunk reads in flight at once while opening the device, and how long before an unanswered
/// one is asked again (the dongle drops a reply now and then; a round trip is ~30 ms).
const READ_WINDOW: usize = 4;
const READ_REASK: Duration = Duration::from_millis(80);
const SHIFT_SLOT: u8 = 5;
const LIGHT_SLOT: u8 = 32;

/// The model whose config interface may be opened: Cyclone 2 and the V2 family, never the
/// read-only Cyclone 2 PID or a recognised-only model.
fn config_model(pid: u16) -> Option<&'static Model> {
    if pid == C2_NON_CONFIG_PID {
        return None;
    }
    models::by_pid(pid).map(|(m, _)| m).filter(|m| !matches!(m.family, Family::Unsupported))
}

/// True if the device is in config mode and may be configured.
pub fn is_config_pid(pid: u16) -> bool {
    config_model(pid).is_some()
}

/// The mode of a table model: the wired/wireless ADC PIDs are fixed, the rest follow the table's flag.
fn mode_of(pid: u16, wireless: bool) -> DeviceMode {
    match pid {
        0x1053 => DeviceMode::WiredAdc,
        0x100B => DeviceMode::WirelessAdc,
        _ if wireless => DeviceMode::Wireless,
        _ => DeviceMode::Wired,
    }
}

/// Every visible GameSir controller. Every table model's config collection can be opened; the Cyclone 2
/// in another mode and every unknown PID appear once (per PID) and are never opened.
pub fn list_devices() -> Result<Vec<DeviceInfo>, Error> {
    with_api(VENDOR_ID, |api| {
        let mut out: Vec<DeviceInfo> = Vec::new();
        for d in api.device_list().filter(|d| d.vendor_id() == VENDOR_ID) {
            let pid = d.product_id();
            let config = is_config_pid(pid);
            if config && d.usage_page() != USAGE_PAGE {
                continue;
            }
            if !config && out.iter().any(|o| o.pid == pid) {
                continue;
            }
            // The USB product string is the emulated pad's ("Xbox 360 Controller for Windows" on the dongle), not the model.
            let (model, name, mode) = if pid == C2_NON_CONFIG_PID {
                ("C2", "GameSir Cyclone 2", DeviceMode::Unsupported)
            } else if let Some((m, wireless)) = models::by_pid(pid) {
                let mode = mode_of(pid, wireless);
                (m.id, m.name, mode)
            } else {
                // Same vendor, unknown PID (a dongle or a model newer than the table).
                ("unknown", "GameSir controller", DeviceMode::Unsupported)
            };
            out.push(DeviceInfo {
                id: d.path().to_string_lossy().into_owned(),
                model: model.into(),
                name: name.into(),
                vid: VENDOR_ID,
                pid,
                mode,
                config_capable: config,
                serial: d.serial_number().filter(|s| !s.is_empty()).map(str::to_owned),
            });
        }
        Ok(out)
    })
}

/// Firmware reply: `10 0a` then UTF-16LE "DDDDDDDD". Chars 1..4 are the device (340 -> 3.4.0), chars 5..8 the dongle (121 -> 1.2.1).
pub fn parse_firmware(r: &[u8]) -> (String, Option<String>) {
    let chars: Vec<char> = r[2..]
        .chunks(2)
        .take(8)
        .map(|c| char::from_u32(u32::from(u16::from_le_bytes([c[0], c[1]]))).unwrap_or('?'))
        .collect();
    let dotted = |s: &[char]| s.iter().map(char::to_string).collect::<Vec<_>>().join(".");
    let device = dotted(chars.get(1..4).unwrap_or(&[]));
    let dongle = chars.get(5..8).unwrap_or(&[]);
    let dongle = (!dongle.iter().all(|&c| c == '0')).then(|| dotted(dongle));
    (device, dongle)
}

/// Device events. `Input` is only produced while test mode is on; `Lost` ends the session.
#[derive(Debug, Clone)]
pub enum Event {
    Input(GamepadState),
    Battery { battery: u8, charging: bool },
    Lost(String),
}

pub type Sink = Arc<dyn Fn(Event) + Send + Sync>;

type Battery = Option<(u8, bool)>;

/// An open, configurable controller. `raw` holds the last full block per slot so codec edits keep unknown bytes.
pub struct Session {
    wire: Wire,
    model: &'static Model,
    writable: bool,
    raw: BTreeMap<u8, Vec<u8>>,
    /// Bytes per slot whose last write got no ack: they may or may not be on the device.
    unsure: BTreeMap<u8, Vec<bool>>,
    testing: Arc<AtomicBool>,
    battery: Arc<Mutex<Battery>>,
    sink: Arc<Mutex<Option<Sink>>>,
    stop: Arc<AtomicBool>,
    vibe_gen: Arc<AtomicU64>,
    reader: Option<thread::JoinHandle<()>>,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Join so the HID handle is released before drop returns; the reader polls `stop` every 10 ms.
        if let Some(handle) = self.reader.take() {
            let _ = handle.join();
        }
    }
}

/// Live input reaches the sink at most this often: a 1000 Hz report stream floods the UI and froze the button test.
const INPUT_EVERY: Duration = Duration::from_millis(16);
/// Reports read per hold of the link lock, so writers still get the link between batches.
const DRAIN_MAX: usize = 64;

/// Sole reader of the HID handle: 0x12 reports become events, everything else goes to `Wire::exchange`.
struct Reader {
    id: String,
    model: &'static Model,
    link: SharedLink,
    replies: Sender<Frame>,
    testing: Arc<AtomicBool>,
    battery: Arc<Mutex<Battery>>,
    sink: Arc<Mutex<Option<Sink>>>,
    stop: Arc<AtomicBool>,
}

impl Reader {
    fn run(self) {
        let mut batch: Vec<(Frame, usize)> = Vec::with_capacity(DRAIN_MAX);
        // Latest input not yet sent, last input sent, and when.
        let mut pending: Option<GamepadState> = None;
        let mut sent: Option<GamepadState> = None;
        let mut sent_at = Instant::now();
        // Buttons seen pressed since the last emit: a tap shorter than one batch or throttle window
        // would otherwise vanish, since only the newest report's axes are kept.
        let mut latched: Vec<String> = Vec::new();
        while !self.stop.load(Ordering::SeqCst) {
            let read = self.drain(&mut batch);
            // Let writers take the link between batches; the mutex is not fair.
            thread::sleep(Duration::from_millis(1));
            for (buf, n) in batch.drain(..) {
                if buf[0] == input::REPORT_ID {
                    if let Some(state) = self.on_input(&buf[..n]) {
                        for b in &state.buttons {
                            if !latched.contains(b) {
                                latched.push(b.clone());
                            }
                        }
                        pending = Some(state);
                    }
                } else if self.replies.send(buf).is_err() {
                    return;
                }
            }
            if let Err(e) = read {
                if !self.stop.load(Ordering::SeqCst) {
                    self.emit(Event::Lost(e.to_string()));
                }
                return;
            }
            if !self.testing.load(Ordering::SeqCst) {
                pending = None;
                sent = None;
                latched.clear();
            } else if pending.is_some() && (sent.is_none() || sent_at.elapsed() >= INPUT_EVERY) {
                let latest = pending.take().unwrap_or_default();
                let mut state = latest.clone();
                for b in latched.drain(..) {
                    if !state.buttons.contains(&b) {
                        state.buttons.push(b);
                    }
                }
                // A latched tap was shown; the release goes out in the next window.
                if state != latest {
                    pending = Some(latest);
                }
                if sent.as_ref() != Some(&state) {
                    sent_at = Instant::now();
                    self.emit(Event::Input(state.clone()));
                    sent = Some(state);
                }
            }
        }
    }

    /// Reads every report already queued (up to `DRAIN_MAX`) under one lock; waits up to 10 ms for the first.
    fn drain(&self, out: &mut Vec<(Frame, usize)>) -> Result<(), Error> {
        let mut link = lock(&self.link);
        let mut timeout = 10;
        while out.len() < DRAIN_MAX {
            let mut buf: Frame = [0; FRAME_LEN];
            let n = link.recv(&mut buf, timeout)?;
            if n == 0 {
                break;
            }
            out.push((buf, n));
            timeout = 0;
        }
        Ok(())
    }

    /// Updates the battery and returns the parsed state when the button test is on.
    fn on_input(&self, report: &[u8]) -> Option<GamepadState> {
        if let Some((battery, charging)) = input::battery_for(self.model, report) {
            let changed = {
                let mut last = lock(&self.battery);
                let next = Some((battery, charging));
                let changed = *last != next;
                *last = next;
                changed
            };
            if changed {
                self.emit(Event::Battery { battery, charging });
            }
        }
        if self.testing.load(Ordering::SeqCst) {
            input::parse_for(self.model, &self.id, report)
        } else {
            None
        }
    }

    fn emit(&self, event: Event) {
        let sink = lock(&self.sink).clone();
        if let Some(sink) = sink {
            sink(event);
        }
    }
}

impl Session {
    pub fn open(id: &str) -> Result<(Session, DeviceSnapshot), Error> {
        let (dev, model) = with_api(VENDOR_ID, |api| {
            let d = api
                .device_list()
                .find(|d| d.path().to_string_lossy() == id && d.usage_page() == USAGE_PAGE)
                .ok_or_else(|| Error::NotFound(id.to_owned()))?;
            let model = config_model(d.product_id()).ok_or_else(|| Error::NotConfigurable(id.to_owned()))?;
            Ok((api.open_path(d.path())?, model))
        })?;
        Self::attach_for(model, id, Box::new(dev))
    }

    /// Cyclone 2 session; see `attach_for`.
    pub fn attach(id: &str, link: Box<dyn Link>) -> Result<(Session, DeviceSnapshot), Error> {
        Self::attach_for(models::by_id("C2").expect("C2 is in the model table"), id, link)
    }

    /// Start the reader, then read every block and the firmware/current-profile replies over `link`.
    pub fn attach_for(model: &'static Model, id: &str, link: Box<dyn Link>) -> Result<(Session, DeviceSnapshot), Error> {
        if matches!(model.family, Family::Unsupported) {
            return Err(Error::Codec(format!("{} cannot be configured yet", model.name)));
        }
        let link = share(link);
        let (replies_tx, replies) = mpsc::channel();
        let testing = Arc::new(AtomicBool::new(false));
        let battery = Arc::new(Mutex::new(None));
        let sink = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let reader = Reader {
            id: id.to_owned(),
            model,
            link: link.clone(),
            replies: replies_tx,
            testing: testing.clone(),
            battery: battery.clone(),
            sink: sink.clone(),
            stop: stop.clone(),
        };
        let reader_thread = thread::spawn(move || reader.run());
        // Built before any fallible step so an early error drops it and stops the reader.
        let mut s = Session {
            wire: Wire { link, replies },
            model,
            writable: matches!(model.family, Family::C2),
            raw: BTreeMap::new(),
            unsure: BTreeMap::new(),
            testing,
            battery,
            sink,
            stop,
            vibe_gen: Arc::new(AtomicU64::new(0)),
            reader: Some(reader_thread),
        };
        s.heartbeat()?;
        let (profile_len, light_len) = block_lengths(model);
        for slot in (1..=SHIFT_SLOT).chain([LIGHT_SLOT]) {
            let len = if slot == LIGHT_SLOT { light_len } else { profile_len };
            let block = s.read_block(slot, len)?;
            s.raw.insert(slot, block);
        }

        let fw = s.wire.exchange(&[0x0f, 0x09], |r| r[1] == 10)?;
        let cur = s.wire.exchange(&[0x0f, 0x0b], |r| r[1] == 12)?;

        let codec = |e: String| Error::Codec(e);
        let battery = *lock(&s.battery);
        let (firmware, dongle_firmware) = parse_firmware(&fw);
        let mut snap = DeviceSnapshot {
            id: id.to_owned(),
            model: model.id.to_owned(),
            firmware,
            dongle_firmware,
            current_profile: cur[2],
            battery: battery.map(|b| b.0),
            charging: battery.map(|b| b.1),
            writable: matches!(model.family, Family::C2),
            ..Default::default()
        };
        for slot in 1..SHIFT_SLOT {
            let p = profile::decode_for(model, &s.raw[&slot]).map_err(|e| codec(e.to_string()))?;
            snap.profiles.push(p);
        }
        snap.shift = Some(profile::decode_for(model, &s.raw[&SHIFT_SLOT]).map_err(|e| codec(e.to_string()))?);
        if let Family::C2 = model.family {
            snap.light = Some(light::decode(&s.raw[&LIGHT_SLOT]).map_err(|e| codec(e.to_string()))?);
        } else {
            snap.effect_light = Some(effect_light::decode(model, &s.raw[&LIGHT_SLOT]).map_err(|e| codec(e.to_string()))?);
        }
        Ok((s, snap))
    }

    /// Receives events from now on; events before the first call are dropped.
    pub fn set_event_sink(&self, sink: Sink) {
        *lock(&self.sink) = Some(sink);
    }

    /// Repeats the current test-mode flag: a plain `HeartBeat(0)` each second switched the button test off.
    pub fn heartbeat(&mut self) -> Result<(), Error> {
        self.wire.send(&command::heartbeat(self.testing.load(Ordering::SeqCst)))
    }

    /// Reads a block with up to `READ_WINDOW` chunk requests in flight. Each request waits a full
    /// wireless round trip, and the dongle drops a reply now and then, so one-at-a-time with a
    /// 200 ms resend made opening the device take ~3 s. Replies are matched by offset; a chunk
    /// unanswered after `READ_REASK` is asked again.
    fn read_block(&mut self, slot: u8, len: usize) -> Result<Vec<u8>, Error> {
        let chunks: Vec<(usize, usize)> = (0..len).step_by(CHUNK).map(|off| (off, (len - off).min(CHUNK))).collect();
        let mut out = vec![0u8; len];
        let mut done = vec![false; chunks.len()];
        let mut asked: Vec<Option<Instant>> = vec![None; chunks.len()];
        let mut tries = vec![0u32; chunks.len()];
        let mut left = chunks.len();
        while left > 0 {
            let mut in_flight = (0..chunks.len()).filter(|&i| asked[i].is_some() && !done[i]).count();
            for (i, &(off, n)) in chunks.iter().enumerate() {
                let stale = asked[i].is_some_and(|t| t.elapsed() >= READ_REASK);
                let fresh = asked[i].is_none() && in_flight < READ_WINDOW;
                if done[i] || !(stale || fresh) {
                    continue;
                }
                // Same total patience as `Wire::exchange` (ATTEMPTS × WAIT).
                if Duration::from_millis(u64::from(tries[i]) * READ_REASK.as_millis() as u64) >= WAIT * ATTEMPTS {
                    return Err(Error::NoReply);
                }
                tries[i] += 1;
                if fresh {
                    in_flight += 1;
                }
                self.wire.send(&pad(&command::read_chunk(slot, off as u16, n as u8)))?;
                asked[i] = Some(Instant::now());
            }
            match self.wire.replies.recv_timeout(Duration::from_millis(20)) {
                Ok(r) if r[1] == 5 && r[2] == slot => {
                    let off = usize::from(u16::from_be_bytes([r[3], r[4]]));
                    let hit = chunks.iter().position(|&(o, n)| o == off && usize::from(r[5]) == n);
                    if let Some(i) = hit.filter(|&i| !done[i]) {
                        let n = chunks[i].1;
                        out[off..off + n].copy_from_slice(&r[6..6 + n]);
                        done[i] = true;
                        left -= 1;
                    }
                }
                Ok(_) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Err(Error::Disconnected),
            }
        }
        Ok(out)
    }

    /// Send only the changed bytes of `new` against the stored block, in ≤58-byte chunks.
    /// The stored copy follows each acknowledged chunk; an unacknowledged chunk may still have
    /// landed, so its bytes are resent by the next save whatever they hold. (The stored copy used
    /// to be dropped on failure, and the next save rewrote every byte from zeros.)
    fn write_block(&mut self, slot: u8, new: Vec<u8>) -> Result<(), Error> {
        let old = self.raw.get(&slot).cloned().unwrap_or_default();
        let mut unsure = self.unsure.remove(&slot).unwrap_or_default();
        unsure.resize(new.len(), false);
        let was_unsure = unsure.clone();
        let differs = |i: usize| old.get(i) != Some(&new[i]) || was_unsure[i];
        // Offsets sent in this call: the only ones a failed readback can leave in doubt.
        let mut touched = vec![false; new.len()];
        let mut i = 0;
        while i < new.len() {
            if !differs(i) {
                i += 1;
                continue;
            }
            let start = i;
            while i < new.len() && i - start < CHUNK && differs(i) {
                i += 1;
            }
            let mut req = vec![0x0f, 0x03, slot];
            req.extend_from_slice(&(start as u16).to_be_bytes());
            req.push((i - start) as u8);
            req.extend_from_slice(&new[start..i]);
            if let Err(e) = self.wire.exchange(&req, |r| r[1] == 6 && r[2] != 1) {
                unsure[start..i].fill(true);
                self.unsure.insert(slot, unsure);
                return Err(e);
            }
            unsure[start..i].fill(false);
            touched[start..i].fill(true);
            if let Some(stored) = self.raw.get_mut(&slot).filter(|b| b.len() == new.len()) {
                stored[start..i].copy_from_slice(&new[start..i]);
            }
        }
        if !touched.contains(&true) {
            self.raw.insert(slot, new);
            return Ok(());
        }
        let back = match self.read_block(slot, new.len()) {
            Ok(b) => b,
            Err(e) => {
                self.unsure.insert(slot, touched);
                return Err(e);
            }
        };
        if back != new {
            self.unsure.insert(slot, (0..new.len()).map(|i| touched[i] && back[i] != new[i]).collect());
            self.raw.insert(slot, back);
            return Err(Error::Verify(slot));
        }
        self.raw.insert(slot, new);
        Ok(())
    }

    /// Writes only the bytes the user's edit changed. The codec canonicalises some disabled
    /// sub-fields (see `cyclone2-profile.md` §12), so the edit is compared with a re-encode of
    /// the stored profile, and only the differing offsets are patched onto the device's raw bytes.
    pub fn save_profile(&mut self, index: u8, profile: &Profile) -> Result<(), Error> {
        self.require_writable()?;
        check_slot(index)?;
        let raw = self.raw.get(&index).cloned().unwrap_or_default();
        let codec = |e: profile::ProfileError| Error::Codec(e.to_string());
        let mut edit = profile.clone();
        edit.raw = raw.clone();
        let stored = profile::decode_for(self.model, &raw).ok();
        // Out-of-range values the edit adds (not ones the device already holds) never reach flash.
        let known = stored.as_ref().map(|s| profile::violations_for(self.model, s)).unwrap_or_default();
        if let Some(bad) = profile::violations_for(self.model, profile).into_iter().find(|v| !known.contains(v)) {
            return Err(Error::Codec(format!("refused to write {bad}")));
        }
        let edited = profile::encode_for(self.model, &edit).map_err(codec)?;
        let Some(stored) = stored.filter(|_| raw.len() == edited.len()) else {
            return self.write_block(index, edited);
        };
        let baseline = profile::encode_for(self.model, &stored).map_err(codec)?;
        let mut bytes = raw;
        for (i, b) in bytes.iter_mut().enumerate() {
            if baseline[i] != edited[i] {
                *b = edited[i];
            }
        }
        self.write_block(index, bytes)
    }

    pub fn save_light(&mut self, light: &LightProfile) -> Result<(), Error> {
        self.require_writable()?;
        if !matches!(self.model.family, Family::C2) {
            return Err(Error::Codec("LightProfile is Cyclone 2 only; use save_effect_light".into()));
        }
        let mut l = light.clone();
        l.raw = self.raw.get(&LIGHT_SLOT).cloned().unwrap_or_default();
        let bytes = light::encode(&l).map_err(|e| Error::Codec(e.to_string()))?;
        self.write_block(LIGHT_SLOT, bytes)
    }

    /// `raw` is not serialized, so encode from the stored block: bits the model does not carry (wake byte bits 1-7) survive.
    pub fn save_effect_light(&mut self, light: &EffectLight) -> Result<(), Error> {
        self.require_writable()?;
        let mut l = light.clone();
        l.raw = self.raw.get(&LIGHT_SLOT).cloned().unwrap_or_default();
        let bytes = effect_light::encode(self.model, &l).map_err(|e| Error::Codec(e.to_string()))?;
        self.write_block(LIGHT_SLOT, bytes)
    }

    /// Fire-and-forget: the switch-profile ack is undocumented (see `docs/protocol/cyclone2-sequencing.md`).
    pub fn switch_profile(&mut self, index: u8) -> Result<(), Error> {
        self.require_writable()?;
        check_slot(index)?;
        self.wire.send(&pad(&[0x0f, 0x07, index]))
    }

    /// Reset a stored profile's `scope` (see `reset::apply`), save it, and return the result.
    pub fn reset_profile(&mut self, index: u8, scope: &str) -> Result<Profile, Error> {
        self.require_writable()?;
        check_slot(index)?;
        let bytes = self.raw.get(&index).ok_or(Error::BadSlot(index))?;
        let mut p = profile::decode_for(self.model, bytes).map_err(|e| Error::Codec(e.to_string()))?;
        let scale = if matches!(self.model.family, Family::V2(_)) { 10 } else { 1 };
        reset::apply_scaled(&mut p, scope, scale).map_err(|e| Error::Reset(e.to_string()))?;
        self.save_profile(index, &p)?;
        Ok(p)
    }

    pub fn set_light_playback(&mut self, playing: bool, frame: u8) -> Result<(), Error> {
        self.wire.send(&pad(&[0x0f, 0x0d, playing as u8, frame]))
    }

    /// Starts the motors and returns at once. A detached timer sends the stop frame unless a newer call replaced it.
    pub fn vibrate(&mut self, left: u8, right: u8, duration_ms: u32) -> Result<(), Error> {
        let duration_ms = duration_ms.min(2000);
        let generation = self.vibe_gen.fetch_add(1, Ordering::SeqCst) + 1;
        self.wire.send(&pad(&[0x0f, 0x20, 0x66, 0x55, left, right]))?;
        let link = self.wire.link.clone();
        let vibe_gen = self.vibe_gen.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(u64::from(duration_ms)));
            // Check under the link lock so a newer start cannot slip between the check and the stop.
            let mut guard = lock(&link);
            if vibe_gen.load(Ordering::SeqCst) == generation {
                let _ = guard.send(&pad(&[0x0f, 0x20, 0x66, 0x55, 0, 0]));
            }
        });
        Ok(())
    }

    pub fn set_test_mode(&mut self, on: bool) -> Result<(), Error> {
        self.testing.store(on, Ordering::SeqCst);
        self.heartbeat()
    }

    fn require_writable(&self) -> Result<(), Error> {
        if self.writable { Ok(()) } else { Err(Error::ReadOnly) }
    }

    /// Lifts the read-only gate of an untested model: codec tests and on-hardware verification only, never the UI.
    #[doc(hidden)]
    pub fn allow_untested_writes(&mut self) {
        self.writable = true;
    }
}

fn check_slot(index: u8) -> Result<(), Error> {
    if (1..=SHIFT_SLOT).contains(&index) {
        Ok(())
    } else {
        Err(Error::BadSlot(index))
    }
}

/// Full profile and light block lengths for a model.
fn block_lengths(model: &Model) -> (usize, usize) {
    match model.family {
        Family::V2(s) => (s.profile_len, s.light_len),
        _ => (PROFILE_LEN, LIGHT_LEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_pids() {
        assert!(is_config_pid(0x101D));
        assert!(is_config_pid(0x100B));
        assert!(!is_config_pid(0x0575));
    }
}
