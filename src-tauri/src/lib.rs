use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;
use std::time::{Duration, Instant};

use opensir_core::device::{self, Session};
use opensir_core::model::{DeviceInfo, DeviceSnapshot, EffectLight, LightProfile, Profile};
use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Emitter, Manager};

struct LiveSession {
    session: Session,
    /// Last `device-state` payload; shared with the event sink.
    state: Arc<Mutex<DeviceState>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DeviceState {
    id: String,
    current_profile: u8,
    battery: Option<u8>,
    charging: Option<bool>,
}

/// Open devices. The map lock is held only to look a session up; device I/O locks that one session.
#[derive(Default)]
struct Sessions(Mutex<HashMap<String, Arc<Mutex<LiveSession>>>>);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DeviceError {
    id: String,
    message: String,
}

/// Runs blocking HID work off the async runtime: a save can wait seconds for acks, and running it
/// inline stalled the runtime workers (and every queued heartbeat) behind one global lock.
async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    spawn_blocking(f).await.map_err(|e| e.to_string())?
}

async fn with_live<T: Send + 'static>(
    app: &AppHandle,
    id: &str,
    f: impl FnOnce(&mut LiveSession) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let sessions = app.state::<Sessions>();
    let live = {
        let map = sessions.0.lock().map_err(|e| e.to_string())?;
        map.get(id).cloned().ok_or_else(|| format!("device {id} is not open"))?
    };
    blocking(move || f(&mut *live.lock().map_err(|e| e.to_string())?)).await
}

async fn with_session<T: Send + 'static>(
    app: &AppHandle,
    id: &str,
    f: impl FnOnce(&mut Session) -> Result<T, opensir_core::transport::Error> + Send + 'static,
) -> Result<T, String> {
    with_live(app, id, move |l| f(&mut l.session).map_err(|e| e.to_string())).await
}

/// Latest enumeration. The hotplug thread refreshes it every second, so `list_devices` can answer
/// from it instead of paying another HID enumeration (~50 ms) while the window starts.
static LAST_LIST: Mutex<Option<(Instant, Vec<DeviceInfo>)>> = Mutex::new(None);

fn enumerate() -> Result<Vec<DeviceInfo>, opensir_core::transport::Error> {
    let list = device::list_devices()?;
    *LAST_LIST.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), list.clone()));
    Ok(list)
}

#[tauri::command]
async fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    if let Some((at, list)) = LAST_LIST.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
        && at.elapsed() < Duration::from_secs(1)
    {
        return Ok(list.clone());
    }
    blocking(|| enumerate().map_err(|e| e.to_string())).await
}

#[tauri::command]
async fn open_device(app: AppHandle, id: String) -> Result<DeviceSnapshot, String> {
    let open_id = id.clone();
    let (session, snapshot) = blocking(move || device::Session::open(&open_id).map_err(|e| e.to_string())).await?;
    let state = Arc::new(Mutex::new(DeviceState {
        id: id.clone(),
        current_profile: snapshot.current_profile,
        battery: snapshot.battery,
        charging: snapshot.charging,
    }));
    session.set_event_sink(event_sink(app.clone(), id.clone(), state.clone()));
    let _ = app.emit("device-state", state.lock().map_err(|e| e.to_string())?.clone());
    let live = Arc::new(Mutex::new(LiveSession { session, state }));
    app.state::<Sessions>().0.lock().map_err(|e| e.to_string())?.insert(id, live);
    Ok(snapshot)
}

/// Session events. Must not take the sessions lock: the reader thread calls this.
fn event_sink(app: AppHandle, id: String, state: Arc<Mutex<DeviceState>>) -> device::Sink {
    Arc::new(move |event| match event {
        device::Event::Input(pad) => {
            let _ = app.emit("gamepad-state", pad);
        }
        device::Event::Battery { battery, charging } => {
            let Ok(mut st) = state.lock() else { return };
            st.battery = Some(battery);
            st.charging = Some(charging);
            let _ = app.emit("device-state", st.clone());
        }
        device::Event::Lost(message) => {
            let app = app.clone();
            let id = id.clone();
            thread::spawn(move || {
                if let Ok(mut map) = app.state::<Sessions>().0.lock() {
                    map.remove(&id);
                }
                let _ = app.emit("device-error", DeviceError { id, message });
            });
        }
    })
}

#[tauri::command]
async fn close_device(app: AppHandle, id: String) -> Result<(), String> {
    let live = app.state::<Sessions>().0.lock().map_err(|e| e.to_string())?.remove(&id);
    // Dropping the session joins its reader thread; keep that off the async runtime.
    blocking(move || {
        drop(live);
        Ok(())
    })
    .await
}

#[tauri::command]
async fn save_profile(app: AppHandle, id: String, index: u8, profile: Profile) -> Result<(), String> {
    with_session(&app, &id, move |s| s.save_profile(index, &profile)).await
}

#[tauri::command]
async fn switch_profile(app: AppHandle, id: String, index: u8) -> Result<(), String> {
    let emitter = app.clone();
    with_live(&app, &id, move |live| {
        live.session.switch_profile(index).map_err(|e| e.to_string())?;
        let mut st = live.state.lock().map_err(|e| e.to_string())?;
        st.current_profile = index;
        let _ = emitter.emit("device-state", st.clone());
        Ok(())
    })
    .await
}

#[tauri::command]
async fn reset_profile(app: AppHandle, id: String, index: u8, scope: String) -> Result<Profile, String> {
    with_session(&app, &id, move |s| s.reset_profile(index, &scope)).await
}

#[tauri::command]
async fn save_light(app: AppHandle, id: String, light: LightProfile) -> Result<(), String> {
    with_session(&app, &id, move |s| s.save_light(&light)).await
}

#[tauri::command]
async fn save_effect_light(app: AppHandle, id: String, light: EffectLight) -> Result<(), String> {
    with_session(&app, &id, move |s| s.save_effect_light(&light)).await
}

#[tauri::command]
async fn set_light_playback(app: AppHandle, id: String, playing: bool, frame: u8) -> Result<(), String> {
    with_session(&app, &id, move |s| s.set_light_playback(playing, frame)).await
}

#[tauri::command]
async fn vibrate(app: AppHandle, id: String, left: u8, right: u8, duration_ms: u32) -> Result<(), String> {
    with_session(&app, &id, move |s| s.vibrate(left, right, duration_ms)).await
}

#[tauri::command]
async fn set_test_mode(app: AppHandle, id: String, on: bool) -> Result<(), String> {
    with_session(&app, &id, move |s| s.set_test_mode(on)).await
}

fn check_json(path: &str) -> Result<(), String> {
    if Path::new(path).extension().is_some_and(|e| e.eq_ignore_ascii_case("json")) {
        Ok(())
    } else {
        Err("profile files must end in .json".into())
    }
}

#[tauri::command]
async fn heartbeat(app: AppHandle, id: String) -> Result<(), String> {
    with_session(&app, &id, |s| s.heartbeat()).await
}

/// `model` is the open device's model id (default "C2"); files are tagged with it.
#[tauri::command]
async fn export_profile(profile: Profile, path: String, model: Option<String>) -> Result<(), String> {
    check_json(&path)?;
    opensir_core::profile::export_json_for(model.as_deref().unwrap_or("C2"), &profile, &path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_profile(path: String, model: Option<String>) -> Result<Profile, String> {
    check_json(&path)?;
    opensir_core::profile::import_json_for(model.as_deref().unwrap_or("C2"), &path).map_err(|e| e.to_string())
}

/// Emits `device-list-changed` only when the enumerated set changes.
fn spawn_hotplug(app: AppHandle) {
    thread::spawn(move || {
        let mut last = String::new();
        loop {
            if let Ok(devices) = enumerate() {
                let now = serde_json::to_string(&devices).unwrap_or_default();
                if now != last {
                    last = now;
                    let _ = app.emit("device-list-changed", devices);
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The first HID enumeration in a process costs ~250 ms (SetupAPI warm-up); pay it while
    // the window and WebView2 are still being created, not when the page asks for the list.
    thread::spawn(|| {
        let _ = enumerate();
    });
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(Sessions::default())
        .setup(|app| {
            spawn_hotplug(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_devices,
            open_device,
            close_device,
            save_profile,
            switch_profile,
            reset_profile,
            save_light,
            save_effect_light,
            set_light_playback,
            vibrate,
            set_test_mode,
            heartbeat,
            export_profile,
            import_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenSir");
}
