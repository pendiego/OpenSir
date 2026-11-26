//! Read-only CLI: `list`, `info <id>`, `dump <id> [--json]`. Never opens unsupported devices.
use std::error::Error;
use std::process::ExitCode;

use opensir_core::device::{Session, list_devices};
use opensir_core::model::DeviceInfo;

fn main() -> ExitCode {
    match run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["list"] => {
            for d in list_devices()? {
                println!("{}  {:04X}:{:04X}  {:?}  {}", d.id, d.vid, d.pid, d.mode, d.name);
            }
            Ok(())
        }
        ["info", id] => {
            let (_, snap) = open_configurable(id)?;
            println!("firmware: {}", snap.firmware);
            println!("current profile: {}", snap.current_profile);
            println!("profiles: {}", snap.profiles.len());
            Ok(())
        }
        ["dump", id] | ["dump", id, "--json"] => {
            let json = args.last().map(String::as_str) == Some("--json");
            let (_, snap) = open_configurable(id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&snap)?);
            } else {
                println!("{snap:#?}");
            }
            Ok(())
        }
        _ => Err("usage: opensir-cli list | info <id> | dump <id> [--json]".into()),
    }
}

/// Refuses unsupported devices before anything is opened.
fn open_configurable(id: &str) -> Result<(Session, opensir_core::model::DeviceSnapshot), Box<dyn Error>> {
    let info: DeviceInfo = list_devices()?
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| format!("no device {id:?}"))?;
    if !info.config_capable {
        return Err(format!("{} is not in config mode (PID {:04X}); refusing to open", info.name, info.pid).into());
    }
    Ok(Session::open(id)?)
}
