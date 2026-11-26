//! HID link, request/reply exchange, and the process-wide HID handle.
//! Framing: `docs/protocol/cyclone2-sequencing.md` (64-byte reports, no report-id prefix).

use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice, HidError};

use crate::command::FRAME_LEN;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hid: {0}")]
    Hid(#[from] HidError),
    #[error("no reply from device")]
    NoReply,
    #[error("device disconnected")]
    Disconnected,
    #[error("device not found: {0}")]
    NotFound(String),
    #[error("device is not in config mode: {0}")]
    NotConfigurable(String),
    #[error("profile slot {0} out of range")]
    BadSlot(u8),
    #[error("codec: {0}")]
    Codec(String),
    #[error("reset: {0}")]
    Reset(String),
    #[error("This model is read-only in OpenSir until it is tested on hardware")]
    ReadOnly,
    #[error("slot {0} did not read back what was written")]
    Verify(u8),
}

/// One 64-byte HID report.
pub type Frame = [u8; FRAME_LEN];

/// A bidirectional report channel. `HidDevice` implements it; tests substitute a fake.
pub trait Link: Send {
    fn send(&mut self, frame: &[u8]) -> Result<(), Error>;
    /// Fill `buf` with one inbound report. Returns 0 on timeout.
    fn recv(&mut self, buf: &mut Frame, timeout_ms: i32) -> Result<usize, Error>;
}

impl Link for HidDevice {
    fn send(&mut self, frame: &[u8]) -> Result<(), Error> {
        HidDevice::write(self, frame)?;
        Ok(())
    }

    fn recv(&mut self, buf: &mut Frame, timeout_ms: i32) -> Result<usize, Error> {
        Ok(HidDevice::read_timeout(self, buf, timeout_ms)?)
    }
}

/// Zero-pad a request to a full 64-byte report.
pub fn pad(bytes: &[u8]) -> Vec<u8> {
    let mut v = bytes.to_vec();
    v.resize(FRAME_LEN, 0);
    v
}

/// The HID handle shared by the reader thread (reads) and senders (writes).
pub type SharedLink = Arc<Mutex<Box<dyn Link>>>;

pub fn share(link: Box<dyn Link>) -> SharedLink {
    Arc::new(Mutex::new(link))
}

/// Write one frame. The lock is held only for the write.
pub fn send_on(link: &SharedLink, frame: &[u8]) -> Result<(), Error> {
    lock(link).send(frame)
}

pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Sending half is shared; config replies arrive from the reader thread over `replies`.
pub struct Wire {
    pub link: SharedLink,
    pub replies: Receiver<Frame>,
}

pub(crate) const ATTEMPTS: u32 = 10;
pub(crate) const WAIT: Duration = Duration::from_millis(200);

impl Wire {
    pub fn send(&self, frame: &[u8]) -> Result<(), Error> {
        send_on(&self.link, frame)
    }

    /// Send `request`, then wait for a config reply that `accept` takes. Resends on silence.
    pub fn exchange(&self, request: &[u8], accept: impl Fn(&Frame) -> bool) -> Result<Frame, Error> {
        let report = pad(request);
        // A late reply to an earlier request must not satisfy this one.
        while self.replies.try_recv().is_ok() {}
        for _ in 0..ATTEMPTS {
            self.send(&report)?;
            let deadline = Instant::now() + WAIT;
            loop {
                let left = deadline.saturating_duration_since(Instant::now());
                if left.is_zero() {
                    break;
                }
                match self.replies.recv_timeout(left) {
                    Ok(f) if accept(&f) => return Ok(f),
                    Ok(_) => {}
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => return Err(Error::Disconnected),
                }
            }
        }
        Err(Error::NoReply)
    }
}

static API: Mutex<Option<HidApi>> = Mutex::new(None);

/// Run `f` with the shared HID handle, after re-listing the devices with vendor id `vid`.
/// A full system enumeration (`refresh_devices`) took ~300 ms and ran every second for hotplug.
pub fn with_api<T>(vid: u16, f: impl FnOnce(&HidApi) -> Result<T, Error>) -> Result<T, Error> {
    let mut guard = API.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(HidApi::new()?);
    }
    let api = guard.as_mut().expect("initialised above");
    api.reset_devices()?;
    api.add_devices(vid, 0)?;
    f(api)
}
